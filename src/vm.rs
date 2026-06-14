use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::Function;
use crate::error::{RuntimeError, StackFrame};
use crate::native;
use crate::obj::{Obj, ObjKind};
use crate::opcode::OpCode;
use crate::upvalue::Upvalue;
use crate::value::Value;

const STACK_MAX: usize = 1024;
const FRAMES_MAX: usize = 256;

struct CallFrame {
    function: Rc<RefCell<Obj>>,
    /// Upvalues captured by this frame's closure. Empty for top-level script
    /// and for non-closure function objects. Indexed by GetUpvalue/SetUpvalue.
    upvalues: Vec<Rc<RefCell<Upvalue>>>,
    ip: usize,
    slots_offset: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    open_upvalues: Vec<Rc<RefCell<Upvalue>>>,
    globals: HashMap<String, Value>,
    pub output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        for (n, f) in native::registry() {
            globals.insert(n.to_string(), Value::Native(f));
        }
        Self {
            frames: Vec::new(),
            stack: Vec::new(),
            open_upvalues: Vec::new(),
            globals,
            output: Vec::new(),
        }
    }

    pub fn run(&mut self, top: Function) -> Result<(), RuntimeError> {
        if top.arity != 0 {
            return Err(RuntimeError {
                message: "top-level function must have arity 0".into(),
                stack_trace: vec![],
            });
        }
        let function_obj = Rc::new(RefCell::new(Obj {
            kind: ObjKind::Function(crate::obj::FunctionObj {
                name: top.name.clone(),
                arity: top.arity,
                chunk: Some(top.chunk.clone()),
                upvalues: top.upvalues.clone(),
            }),
        }));
        // Push a synthetic frame.
        self.frames.push(CallFrame {
            function: function_obj,
            upvalues: Vec::new(),
            ip: 0,
            slots_offset: 0,
        });
        // Slot 0 holds the closure itself.
        self.stack.push(Value::Obj(self.frames[0].function.clone()));
        self.execute()
    }

    fn frame_chunk(&self) -> std::cell::Ref<'_, crate::opcode::Chunk> {
        let fr = self.frames.last().unwrap();
        std::cell::Ref::map(fr.function.borrow(), |o| {
            match &o.kind {
                ObjKind::Function(fo) => fo.chunk.as_ref().unwrap(),
                _ => unreachable!(),
            }
        })
    }

    fn frame_function_obj(&self) -> std::cell::Ref<'_, Obj> {
        let fr = self.frames.last().unwrap();
        fr.function.borrow()
    }

    fn frame_name(&self) -> String {
        let f = self.frame_function_obj();
        match &f.kind {
            ObjKind::Function(fo) => fo.name.clone(),
            _ => "<script>".into(),
        }
    }

    fn line_at(&self, ip: usize) -> usize {
        if let Some(fr) = self.frames.last() {
            let f = fr.function.borrow();
            if let ObjKind::Function(fo) = &f.kind {
                if let Some(chunk) = &fo.chunk {
                    return chunk.lines.get(ip).copied().unwrap_or(0);
                }
            }
        }
        0
    }

    fn read_byte(&mut self) -> u8 {
        let ip = self.frames.last().unwrap().ip;
        let b = self.frame_chunk().code[ip];
        self.frames.last_mut().unwrap().ip += 1;
        b
    }

    fn read_u16(&mut self) -> u16 {
        let hi = self.read_byte() as u16;
        let lo = self.read_byte() as u16;
        (hi << 8) | lo
    }

    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte() as usize;
        self.frame_chunk().constants[idx].clone()
    }

    fn read_short_string(&mut self) -> String {
        let v = self.read_constant();
        match v {
            Value::Obj(o) => {
                let b = o.borrow();
                if let ObjKind::String(s) = &b.kind {
                    s.clone()
                } else {
                    panic!("expected string constant")
                }
            }
            _ => panic!("expected string constant"),
        }
    }

    fn push(&mut self, v: Value) {
        if self.stack.len() >= STACK_MAX {
            panic!("stack overflow");
        }
        self.stack.push(v);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("pop on empty stack")
    }

    fn peek(&self, n: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - n]
    }

    fn capture_upvalue(&mut self, slot: usize) -> Rc<RefCell<Upvalue>> {
        for uv in &self.open_upvalues {
            let b = uv.borrow();
            if b.index == slot && b.is_open {
                return uv.clone();
            }
        }
        let uv = Rc::new(RefCell::new(Upvalue::new(slot, true)));
        self.open_upvalues.push(uv.clone());
        uv
    }

    fn close_upvalues(&mut self, from: usize) {
        for uv in &self.open_upvalues {
            let mut b = uv.borrow_mut();
            if b.index >= from && b.is_open {
                b.value = self.stack[b.index].clone();
                b.is_open = false;
            }
        }
        // Reclaim closed upvalues: drop any whose stack slot has been truncated.
        self.open_upvalues.retain(|uv| {
            let b = uv.borrow();
            b.is_open || b.index < from
        });
    }

    fn runtime_err(&self, msg: &str) -> RuntimeError {
        let mut trace = vec![];
        for (i, fr) in self.frames.iter().enumerate().rev() {
            let f = fr.function.borrow();
            let (name, line) = match &f.kind {
                ObjKind::Function(fo) => {
                    let ln = fo
                        .chunk
                        .as_ref()
                        .and_then(|c| c.lines.get(fr.ip.saturating_sub(1)).copied())
                        .unwrap_or(0);
                    (fo.name.clone(), ln)
                }
                _ => ("<script>".into(), 0),
            };
            trace.push(StackFrame { function: name, line });
            if i == 0 {
                break;
            }
        }
        RuntimeError {
            message: msg.to_string(),
            stack_trace: trace,
        }
    }

    fn execute(&mut self) -> Result<(), RuntimeError> {
        loop {
            let op_byte = self.read_byte();
            let op = match OpCode::from_u8(op_byte) {
                Some(o) => o,
                None => return Err(self.runtime_err(&format!("unknown opcode {}", op_byte))),
            };
            match op {
                OpCode::Constant => {
                    let v = self.read_constant();
                    self.push(v);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    let off = self.frames.last().unwrap().slots_offset;
                    let v = self.stack[off + slot].clone();
                    self.push(v);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let off = self.frames.last().unwrap().slots_offset;
                    self.stack[off + slot] = self.peek(0).clone();
                }
                OpCode::GetGlobal => {
                    let name = self.read_short_string();
                    let v = self.globals.get(&name).cloned().ok_or_else(|| {
                        self.runtime_err(&format!("undefined variable '{}'", name))
                    })?;
                    self.push(v);
                }
                OpCode::DefineGlobal => {
                    let name = self.read_short_string();
                    let v = self.pop();
                    self.globals.insert(name, v);
                }
                OpCode::SetGlobal => {
                    let name = self.read_short_string();
                    let v = self.peek(0).clone();
                    if let Some(slot) = self.globals.get_mut(&name) {
                        *slot = v;
                    } else {
                        return Err(self.runtime_err(&format!(
                            "undefined variable '{}'",
                            name
                        )));
                    }
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let fr = self.frames.last().unwrap();
                    let uv = fr.upvalues[slot].clone();
                    let v = {
                        let b = uv.borrow();
                        if b.is_open {
                            self.stack[b.index].clone()
                        } else {
                            b.value.clone()
                        }
                    };
                    self.push(v);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let fr = self.frames.last().unwrap();
                    let uv = fr.upvalues[slot].clone();
                    let v_peek = self.peek(0).clone();
                    let mut b = uv.borrow_mut();
                    if b.is_open {
                        self.stack[b.index] = v_peek;
                    } else {
                        b.value = v_peek;
                    }
                }
                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.add(a, b)?);
                }
                OpCode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(num_bin(a, b, |x, y| x - y)?);
                }
                OpCode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(num_bin(a, b, |x, y| x * y)?);
                }
                OpCode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(num_bin(a, b, |x, y| x / y)?);
                }
                OpCode::Neg => {
                    let v = self.pop();
                    if let Value::Number(n) = v {
                        self.push(Value::Number(-n));
                    } else {
                        return Err(self.runtime_err("operand must be a number"));
                    }
                }
                OpCode::Not => {
                    let v = self.pop();
                    self.push(Value::Bool(!v.is_truthy()));
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a.equals(&b)));
                }
                OpCode::Greater => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(num_cmp(a, b, |x, y| x > y)?);
                }
                OpCode::Less => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(num_cmp(a, b, |x, y| x < y)?);
                }
                OpCode::Print => {
                    let v = self.pop();
                    self.output.push(v.to_string());
                }
                OpCode::Jump => {
                    let off = self.read_u16() as usize;
                    self.frames.last_mut().unwrap().ip += off;
                }
                OpCode::JumpIfFalse => {
                    let off = self.read_u16() as usize;
                    if !self.peek(0).is_truthy() {
                        self.frames.last_mut().unwrap().ip += off;
                    }
                }
                OpCode::Loop => {
                    let off = self.read_u16() as usize;
                    self.frames.last_mut().unwrap().ip -= off;
                }
                OpCode::Call => {
                    let argc = self.read_byte() as usize;
                    self.call_value(argc)?;
                }
                OpCode::Closure => {
                    let v = self.read_constant();
                    if let Value::Obj(o) = v {
                        let captured_obj = o.borrow();
                        let fo = match &captured_obj.kind {
                            ObjKind::Function(fo) => fo.clone(),
                            _ => {
                                return Err(self.runtime_err("CLOSURE expected function"));
                            }
                        };
                        drop(captured_obj);

                        // Read upvalue operands.
                        let n_upvals = fo.upvalues.len();
                        let mut upvalues: Vec<Rc<RefCell<Upvalue>>> = Vec::with_capacity(n_upvals);
                        for _ in 0..n_upvals {
                            let is_local = self.read_byte() != 0;
                            let idx = self.read_byte() as usize;
                            if is_local {
                                let slot = self.frames.last().unwrap().slots_offset + idx;
                                upvalues.push(self.capture_upvalue(slot));
                            } else {
                                // Non-local upvalue from enclosing closure.
                                let fr = self.frames.last().unwrap();
                                let f = fr.function.borrow();
                                let enclosing_fo = match &f.kind {
                                    ObjKind::Function(fo) => fo,
                                    _ => unreachable!(),
                                };
                                let enc_uv = enclosing_fo.upvalues[idx].clone();
                                drop(f);
                                // Find or create a shared Rc<RefCell<Upvalue>>.
                                let shared = if let Some(existing) = self
                                    .open_upvalues
                                    .iter()
                                    .find(|uv| {
                                        let b = uv.borrow();
                                        b.index == enc_uv.index as usize
                                            && b.is_open
                                            && /* is_local flag */ enc_uv.is_local
                                    })
                                    .cloned()
                                {
                                    existing
                                } else {
                                    // Take the value from the enclosing stack slot.
                                    let val = self.stack
                                        [self.frames.last().unwrap().slots_offset + enc_uv.index as usize]
                                        .clone();
                                    let closed = Rc::new(RefCell::new(Upvalue {
                                        index: enc_uv.index as usize,
                                        is_open: false,
                                        value: val,
                                    }));
                                    closed
                                };
                                upvalues.push(shared);
                            }
                        }

                        let closure = Obj {
                            kind: ObjKind::Closure(crate::obj::ClosureObj {
                                function: Box::new(crate::obj::FunctionObj {
                                    name: fo.name.clone(),
                                    arity: fo.arity,
                                    chunk: fo.chunk.clone(),
                                    upvalues: fo.upvalues.clone(),
                                }),
                                upvalues,
                            }),
                        };
                        self.push(Value::Obj(Rc::new(RefCell::new(closure))));
                    } else {
                        return Err(self.runtime_err("CLOSURE expected function"));
                    }
                }
                OpCode::CloseUpvalue => {
                    let slot = self.stack.len() - 1;
                    self.close_upvalues(slot);
                    self.pop();
                }
                OpCode::Return => {
                    let result = self.pop();
                    let frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        return Ok(());
                    }
                    self.close_upvalues(frame.slots_offset);
                    self.stack.truncate(frame.slots_offset);
                    self.push(result);
                }
                OpCode::Class
                | OpCode::Inherit
                | OpCode::Method
                | OpCode::GetProperty
                | OpCode::SetProperty
                | OpCode::Invoke
                | OpCode::SuperInvoke => {
                    return Err(self.runtime_err(&format!(
                        "opcode {:?} not implemented in v1",
                        op
                    )));
                }
            }
        }
    }

    fn add(&self, a: Value, b: Value) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (Value::Obj(o1), Value::Obj(o2)) => {
                let (a, b) = (o1.borrow(), o2.borrow());
                if let (ObjKind::String(s1), ObjKind::String(s2)) = (&a.kind, &b.kind) {
                    Ok(Value::from_string(format!("{}{}", s1, s2)))
                } else {
                    Err(self.runtime_err(
                        "operands must be two numbers or two strings",
                    ))
                }
            }
            _ => Err(self.runtime_err(
                "operands must be two numbers or two strings",
            )),
        }
    }

    fn call_value(&mut self, argc: usize) -> Result<(), RuntimeError> {
        let callee = self.peek(argc).clone();
        match callee {
            Value::Obj(o) => {
                let kind = o.borrow().kind.clone();
                match kind {
                    ObjKind::Closure(co) => {
                        if co.function.arity as usize != argc {
                            return Err(self.runtime_err(&format!(
                                "expected {} args, got {}",
                                co.function.arity, argc
                            )));
                        }
                        if self.frames.len() + 1 >= FRAMES_MAX {
                            return Err(self.runtime_err("stack overflow"));
                        }
                        let new_frame_obj = Rc::new(RefCell::new(Obj {
                            kind: ObjKind::Function(crate::obj::FunctionObj {
                                name: co.function.name.clone(),
                                arity: co.function.arity,
                                chunk: co.function.chunk.clone(),
                                upvalues: co.function.upvalues.clone(),
                            }),
                        }));
                        self.frames.push(CallFrame {
                            function: new_frame_obj,
                            upvalues: co.upvalues.clone(),
                            ip: 0,
                            slots_offset: self.stack.len() - argc - 1,
                        });
                        // Pre-fill arg slots (callee already at offset; we may have
                        // leftover slots). The compiler should already have
                        // emitted GetLocal / etc. for args, so we don't need to
                        // copy them here. Slots beyond argc are uninitialized
                        // (we use lenient indexing; if the function reads them
                        // before defining, it's a bug, but the compiler won't
                        // emit that).
                        Ok(())
                    }
                    _ => Err(self.runtime_err("not a function")),
                }
            }
            Value::Native(f) => {
                let args: Vec<Value> = self.stack[self.stack.len() - argc..].to_vec();
                let r = f(&args).map_err(|m| self.runtime_err(&m))?;
                self.stack.truncate(self.stack.len() - argc - 1);
                self.push(r);
                Ok(())
            }
            _ => Err(self.runtime_err("not callable")),
        }
    }
}

fn num_bin<F: Fn(f64, f64) -> f64>(a: Value, b: Value, f: F) -> Result<Value, RuntimeError> {
    if let (Value::Number(x), Value::Number(y)) = (a, b) {
        Ok(Value::Number(f(x, y)))
    } else {
        Err(RuntimeError {
            message: "operands must be numbers".into(),
            stack_trace: vec![],
        })
    }
}

fn num_cmp<F: Fn(f64, f64) -> bool>(a: Value, b: Value, f: F) -> Result<Value, RuntimeError> {
    if let (Value::Number(x), Value::Number(y)) = (a, b) {
        Ok(Value::Bool(f(x, y)))
    } else {
        Err(RuntimeError {
            message: "operands must be numbers".into(),
            stack_trace: vec![],
        })
    }
}
