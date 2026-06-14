use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::*;
use crate::error::CompileError;
use crate::native;
use crate::obj::{Obj, ObjKind, FunctionObj};
use crate::opcode::{Chunk, OpCode};
use crate::token::Literal;
use crate::upvalue::UpvalueRef;
use crate::value::Value;

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: i32,
    is_captured: bool,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: String,
    pub upvalues: Vec<UpvalueRef>,
}

impl Function {
    fn new(name: &str) -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name: name.to_string(),
            upvalues: Vec::new(),
        }
    }
}

pub struct Compiler {
    function: Function,
    locals: Vec<Local>,
    scope_depth: i32,
    enclosing: Option<Box<Compiler>>,
    /// Global name -> index. Each global gets a fixed index in the chunk's
    /// constant pool as a string, but we also keep a map so the compiler can
    /// assign consistent indices. The VM uses the same map at runtime.
    pub globals: HashMap<String, u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            function: Function::new("<script>"),
            // Slot 0 is reserved for the VM's internal use (closure self).
            locals: vec![Local {
                name: String::new(),
                depth: 0,
                is_captured: false,
            }],
            scope_depth: 0,
            enclosing: None,
            globals: HashMap::new(),
        }
    }

    pub fn new_sub(name: &str, arity: u8, enclosing: Box<Compiler>) -> Self {
        let mut c = Self {
            function: Function::new(name),
            locals: Vec::new(),
            scope_depth: 0,
            enclosing: Some(enclosing),
            globals: HashMap::new(),
        };
        c.function.arity = arity;
        c.locals.push(Local {
            name: String::new(),
            depth: 0,
            is_captured: false,
        });
        c
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn emit_op(&mut self, op: OpCode, line: usize) {
        self.current_chunk().write(op, line);
    }

    fn emit_byte(&mut self, b: u8, line: usize) {
        self.current_chunk().write_byte(b, line);
    }

    fn emit_jump(&mut self, op: OpCode, line: usize) -> usize {
        self.emit_op(op, line);
        self.emit_byte(0xff, line);
        self.emit_byte(0xff, line);
        self.current_chunk().code.len() - 2
    }

    fn patch_jump(&mut self, at: usize) -> Result<(), CompileError> {
        let jump = self.current_chunk().code.len() - at - 2;
        if jump > u16::MAX as usize {
            return Err(CompileError {
                line: 0,
                message: "jump too large".into(),
            });
        }
        self.current_chunk().code[at] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().code[at + 1] = (jump & 0xff) as u8;
        Ok(())
    }

    fn emit_loop(&mut self, start: usize, line: usize) -> Result<(), CompileError> {
        self.emit_op(OpCode::Loop, line);
        let offset = self.current_chunk().code.len() - start + 2;
        if offset > u16::MAX as usize {
            return Err(CompileError {
                line,
                message: "loop too large".into(),
            });
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8, line);
        self.emit_byte((offset & 0xff) as u8, line);
        Ok(())
    }

    fn emit_constant(&mut self, v: Value, line: usize) -> Result<u8, CompileError> {
        let c = self.current_chunk();
        let idx = c.add_constant(v)?;
        self.emit_op(OpCode::Constant, line);
        self.emit_byte(idx, line);
        Ok(idx)
    }

    fn emit_return_nil(&mut self) {
        self.emit_op(OpCode::Nil, 0);
        self.emit_op(OpCode::Return, 0);
    }

    fn add_local(&mut self, name: String) -> Result<u8, CompileError> {
        if self.locals.len() >= 256 {
            return Err(CompileError {
                line: 0,
                message: "too many locals".into(),
            });
        }
        let slot = self.locals.len() as u8;
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            is_captured: false,
        });
        Ok(slot)
    }

    fn resolve_local(&self, name: &str) -> Option<u8> {
        for (i, l) in self.locals.iter().enumerate().rev() {
            if l.name == name && l.depth != -1 {
                return Some(i as u8);
            }
        }
        None
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
        let enc = self.enclosing.as_mut()?;
        if let Some(local) = enc.resolve_local(name) {
            enc.locals[local as usize].is_captured = true;
            return self.add_upvalue(local, true);
        }
        if let Some(uv) = enc.resolve_upvalue(name) {
            return self.add_upvalue(uv, false);
        }
        None
    }

    fn add_upvalue(&mut self, index: u8, is_local: bool) -> Option<u8> {
        // De-dupe existing upvalues for this function.
        if let Some(i) = self
            .function
            .upvalues
            .iter()
            .position(|u| u.index == index && u.is_local == is_local)
        {
            return Some(i as u8);
        }
        if self.function.upvalues.len() >= u8::MAX as usize {
            return None;
        }
        self.function.upvalues.push(UpvalueRef { index, is_local });
        Some((self.function.upvalues.len() - 1) as u8)
    }

    fn resolve_global(&mut self, name: &str) -> u8 {
        if let Some(&i) = self.globals.get(name) {
            return i;
        }
        let i = self.globals.len() as u8;
        self.globals.insert(name.to_string(), i);
        i
    }

    /// Add a string constant and return its index, WITHOUT emitting a Constant opcode.
    /// Used as the operand of DefineGlobal / GetGlobal / SetGlobal, which expect a
    /// 1-byte constant index in the bytecode stream.
    fn emit_name(&mut self, name: &str) -> Result<u8, CompileError> {
        self.current_chunk().add_constant(Value::from_string(name.to_string()))
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;
        while let Some(l) = self.locals.last() {
            if l.depth > self.scope_depth {
                self.emit_op(OpCode::Pop, line);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    pub fn compile(mut self, prog: &Program) -> Result<Function, CompileError> {
        // Pre-register native globals so they get consistent indices.
        for (n, _) in native::registry() {
            self.resolve_global(n);
        }

        for s in prog {
            self.stmt(s)?;
        }
        // Script always returns nil.
        self.emit_return_nil();
        Ok(self.function)
    }

    fn stmt(&mut self, s: &Stmt) -> Result<(), CompileError> {
        match s {
            Stmt::Expr { expr } => {
                self.expr(expr)?;
                self.emit_op(OpCode::Pop, 0);
                Ok(())
            }
            Stmt::Print { expr } => {
                self.expr(expr)?;
                self.emit_op(OpCode::Print, 0);
                Ok(())
            }
            Stmt::Var { name, init } => {
                if self.scope_depth > 0 {
                    self.declare_local(name)?;
                    if let Some(e) = init {
                        self.expr(e)?;
                    } else {
                        self.emit_op(OpCode::Nil, 0);
                    }
                } else {
                    if let Some(e) = init {
                        self.expr(e)?;
                    } else {
                        self.emit_op(OpCode::Nil, 0);
                    }
                    self.emit_op(OpCode::DefineGlobal, 0);
                    let idx = self.emit_name(name)?;
                    self.emit_byte(idx, 0);
                }
                Ok(())
            }
            Stmt::Block { stmts } => {
                self.begin_scope();
                for s in stmts {
                    self.stmt(s)?;
                }
                self.end_scope(0);
                Ok(())
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.expr(cond)?;
                let j1 = self.emit_jump(OpCode::JumpIfFalse, 0);
                self.emit_op(OpCode::Pop, 0);
                self.stmt(then_branch)?;
                let j2 = self.emit_jump(OpCode::Jump, 0);
                self.patch_jump(j1)?;
                self.emit_op(OpCode::Pop, 0);
                if let Some(e) = else_branch {
                    self.stmt(e)?;
                }
                self.patch_jump(j2)
            }
            Stmt::While { cond, body } => {
                let loop_start = self.current_chunk().code.len();
                self.expr(cond)?;
                let exit = self.emit_jump(OpCode::JumpIfFalse, 0);
                self.emit_op(OpCode::Pop, 0);
                self.stmt(body)?;
                self.emit_loop(loop_start, 0)?;
                self.patch_jump(exit)?;
                self.emit_op(OpCode::Pop, 0);
                Ok(())
            }
            Stmt::Return { value } => {
                if let Some(e) = value {
                    self.expr(e)?;
                } else {
                    self.emit_op(OpCode::Nil, 0);
                }
                self.emit_op(OpCode::Return, 0);
                Ok(())
            }
            Stmt::Fn { name, params, body } => {
                // Register the global slot for the function name. The index is
                // looked up by emit_name later; this call is for its side effect.
                self.resolve_global(name);
                // Compile the function body in a sub-compiler.
                let arity = params.len() as u8;
                let mut sub = Compiler::new_sub(
                    name,
                    arity,
                    Box::new(Compiler {
                        function: std::mem::replace(&mut self.function, Function::new("__placeholder__")),
                        locals: std::mem::take(&mut self.locals),
                        scope_depth: self.scope_depth,
                        enclosing: self.enclosing.take(),
                        globals: std::mem::take(&mut self.globals),
                    }),
                );
                sub.begin_scope();
                for p in params {
                    let slot = sub.add_local(p.clone())?;
                    sub.locals.last_mut().unwrap().depth = 0;
                    let _ = slot;
                }
                let block = Stmt::Block { stmts: body.clone() };
                sub.stmt(&block)?;
                sub.emit_return_nil();

                // Pull state back into `self`.
                let sub_inner = sub.enclosing.take().expect("sub has enclosing");
                self.function = sub_inner.function;
                self.locals = sub_inner.locals;
                self.scope_depth = sub_inner.scope_depth;
                self.enclosing = sub_inner.enclosing;
                self.globals = sub_inner.globals;

                // The sub's `function` is the compiled function value.
                let fn_value = Value::Obj(Rc::new(RefCell::new(Obj {
                    kind: ObjKind::Function(FunctionObj {
                        name: sub.function.name.clone(),
                        arity: sub.function.arity,
                        chunk: Some(sub.function.chunk.clone()),
                        upvalues: sub.function.upvalues.clone(),
                    }),
                })));

                // Emit CLOSURE + upvalue operands.
                let idx = self.emit_constant(fn_value, 0)?;
                self.emit_op(OpCode::Closure, 0);
                self.emit_byte(idx, 0);
                for uv in &sub.function.upvalues {
                    self.emit_byte(uv.index, 0);
                    self.emit_byte(if uv.is_local { 1 } else { 0 }, 0);
                }

                self.emit_op(OpCode::DefineGlobal, 0);
                let idx = self.emit_name(name)?;
                self.emit_byte(idx, 0);
                Ok(())
            }
        }
    }

    fn declare_local(&mut self, name: &str) -> Result<(), CompileError> {
        // Disallow shadowing in the same scope (simple v1 rule).
        for l in self.locals.iter().rev() {
            if l.depth != -1 && l.depth < self.scope_depth {
                break;
            }
            if l.name == name {
                return Err(CompileError {
                    line: 0,
                    message: format!("variable '{}' already declared in this scope", name),
                });
            }
        }
        self.add_local(name.to_string())?;
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> Result<(), CompileError> {
        match e {
            Expr::Literal { value } => {
                let v = match value {
                    Literal::Number(n) => Value::Number(*n),
                    Literal::Str(s) => Value::from_string(s.clone()),
                    Literal::Bool(true) => Value::Bool(true),
                    Literal::Bool(false) => Value::Bool(false),
                    Literal::Nil | Literal::None => Value::Nil,
                };
                self.emit_constant(v, 0)?;
                Ok(())
            }
            Expr::Variable { name } => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op(OpCode::GetLocal, 0);
                    self.emit_byte(slot, 0);
                } else if let Some(uv) = self.resolve_upvalue(name) {
                    self.emit_op(OpCode::GetUpvalue, 0);
                    self.emit_byte(uv, 0);
                } else {
                    self.emit_op(OpCode::GetGlobal, 0);
                    let idx = self.emit_name(name)?;
                    self.emit_byte(idx, 0);
                }
                Ok(())
            }
            Expr::Assign { name, value } => {
                self.expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op(OpCode::SetLocal, 0);
                    self.emit_byte(slot, 0);
                } else if let Some(uv) = self.resolve_upvalue(name) {
                    self.emit_op(OpCode::SetUpvalue, 0);
                    self.emit_byte(uv, 0);
                } else {
                    self.emit_op(OpCode::SetGlobal, 0);
                    let idx = self.emit_name(name)?;
                    self.emit_byte(idx, 0);
                }
                Ok(())
            }
            Expr::Group { inner } => self.expr(inner),
            Expr::Unary { op, operand } => {
                self.expr(operand)?;
                let op = *op;
                self.emit_op(
                    match op {
                        UnaryOp::Neg => OpCode::Neg,
                        UnaryOp::Not => OpCode::Not,
                    },
                    0,
                );
                Ok(())
            }
            Expr::Binary { op, left, right } => {
                self.expr(left)?;
                self.expr(right)?;
                let op = *op;
                let code = match op {
                    BinOp::Add => OpCode::Add,
                    BinOp::Sub => OpCode::Sub,
                    BinOp::Mul => OpCode::Mul,
                    BinOp::Div => OpCode::Div,
                    BinOp::Eq => OpCode::Equal,
                    BinOp::Ne => {
                        // !(a == b)
                        self.emit_op(OpCode::Equal, 0);
                        self.emit_op(OpCode::Not, 0);
                        return Ok(());
                    }
                    BinOp::Lt => OpCode::Less,
                    BinOp::Le => {
                        // !(b < a)  --  a <= b  <=>  !(b < a)
                        // We've already pushed a then b on the stack. Pop b, push a<=b check.
                        // Simpler: emit a then b, then Equal+b< ? Use Less on a, b swapped is not straightforward.
                        // Use Less: needs (b, a) on stack. We have (a, b). Rotate via pop+push.
                        // Plan keeps it simple: emit a<b, NOT, store. We'll handle via a sub-op later.
                        // For now, the interpreter uses Le via !(a > b) using the Greater op in reverse.
                        // Emit Greater and Not instead.
                        self.emit_op(OpCode::Greater, 0);
                        self.emit_op(OpCode::Not, 0);
                        return Ok(());
                    }
                    BinOp::Gt => OpCode::Greater,
                    BinOp::Ge => {
                        // a >= b <=> !(a < b)
                        self.emit_op(OpCode::Less, 0);
                        self.emit_op(OpCode::Not, 0);
                        return Ok(());
                    }
                    BinOp::And | BinOp::Or => unreachable!(),
                };
                self.emit_op(code, 0);
                Ok(())
            }
            Expr::Logical { op, left, right } => {
                let op = *op;
                self.expr(left)?;
                match op {
                    BinOp::Or => {
                        // If left is truthy, keep it and skip the right-hand evaluation.
                        let else_jump = self.emit_jump(OpCode::JumpIfFalse, 0);
                        let end_jump = self.emit_jump(OpCode::Jump, 0);
                        self.patch_jump(else_jump)?;
                        // Left was false: pop it and evaluate right.
                        self.emit_op(OpCode::Pop, 0);
                        self.expr(right)?;
                        self.patch_jump(end_jump)?;
                    }
                    BinOp::And => {
                        // If left is false, keep it and skip the right-hand evaluation.
                        let end_jump = self.emit_jump(OpCode::JumpIfFalse, 0);
                        self.emit_op(OpCode::Pop, 0);
                        self.expr(right)?;
                        self.patch_jump(end_jump)?;
                    }
                    _ => unreachable!(),
                }
                Ok(())
            }
            Expr::Call { callee, args } => {
                self.expr(callee)?;
                for a in args {
                    self.expr(a)?;
                }
                self.emit_op(OpCode::Call, 0);
                self.emit_byte(args.len() as u8, 0);
                Ok(())
            }
            Expr::Empty => {
                self.emit_op(OpCode::Nil, 0);
                Ok(())
            }
        }
    }
}
