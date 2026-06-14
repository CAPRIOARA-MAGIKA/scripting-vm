use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::*;
use crate::env::Environment;
use crate::error::RuntimeError;
use crate::obj::ObjKind;
use crate::token::Literal;
use crate::value::Value;

pub type OutputSink = Rc<RefCell<Vec<String>>>;

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    output: OutputSink,
}

impl Interpreter {
    pub fn with_sink(output: OutputSink) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        Self { globals, output }
    }

    pub fn output(&self) -> OutputSink {
        self.output.clone()
    }

    pub fn globals(&self) -> Rc<RefCell<Environment>> {
        self.globals.clone()
    }

    pub fn run(&self, prog: &Program) -> Result<(), RuntimeError> {
        for s in prog {
            self.exec_stmt(s, self.globals.clone())?;
        }
        Ok(())
    }

    fn exec_stmt(
        &self,
        s: &Stmt,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        match s {
            Stmt::Expr { expr } => {
                self.eval(expr, env)?;
                Ok(())
            }
            Stmt::Print { expr } => {
                let v = self.eval(expr, env)?;
                self.output.borrow_mut().push(v.to_string());
                Ok(())
            }
            Stmt::Var { name, init } => {
                let v = if let Some(e) = init {
                    self.eval(e, env.clone())?
                } else {
                    Value::Nil
                };
                env.borrow_mut().define(name.clone(), v);
                Ok(())
            }
            Stmt::Block { stmts } => {
                let child = Rc::new(RefCell::new(Environment::new_child(env)));
                for s in stmts {
                    self.exec_stmt(s, child.clone())?;
                }
                Ok(())
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.eval(cond, env.clone())?.is_truthy() {
                    self.exec_stmt(then_branch, env)?;
                } else if let Some(e) = else_branch {
                    self.exec_stmt(e, env)?;
                }
                Ok(())
            }
            Stmt::While { cond, body } => {
                while self.eval(cond, env.clone())?.is_truthy() {
                    self.exec_stmt(body, env.clone())?;
                }
                Ok(())
            }
            Stmt::Return { value } => {
                let v = if let Some(e) = value {
                    Some(self.eval(e, env)?)
                } else {
                    None
                };
                Err(RuntimeError {
                    message: format!("return {:?}", v),
                    stack_trace: vec![],
                })
            }
            Stmt::Fn { .. } => Ok(()),
        }
    }

    fn eval(
        &self,
        e: &Expr,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Value, RuntimeError> {
        match e {
            Expr::Literal { value } => Ok(self.from_literal(value.clone())),
            Expr::Variable { name } => env
                .borrow()
                .get(name)
                .ok_or_else(|| RuntimeError {
                    message: format!("undefined variable '{}'", name),
                    stack_trace: vec![],
                }),
            Expr::Assign { name, value } => {
                let v = self.eval(value, env.clone())?;
                env.borrow_mut().assign(name, v.clone())?;
                Ok(v)
            }
            Expr::Group { inner } => self.eval(inner, env.clone()),
            Expr::Unary { op, operand } => {
                let op = *op;
                let v = self.eval(operand, env.clone())?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError {
                            message: "operand must be a number".into(),
                            stack_trace: vec![],
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                }
            }
            Expr::Binary { op, left, right } => {
                let op = *op;
                self.eval_binary(op, left, right, env.clone())
            }
            Expr::Logical { op, left, right } => {
                let op = *op;
                let l = self.eval(left, env.clone())?;
                match op {
                    BinOp::And => {
                        if !l.is_truthy() {
                            Ok(Value::Bool(false))
                        } else {
                            Ok(Value::Bool(self.eval(right, env)?.is_truthy()))
                        }
                    }
                    BinOp::Or => {
                        if l.is_truthy() {
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(self.eval(right, env)?.is_truthy()))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Call { callee, args } => {
                let callee_v = self.eval(callee, env.clone())?;
                let _: Result<Vec<Value>, RuntimeError> =
                    args.iter().map(|a| self.eval(a, env.clone())).collect();
                match callee_v {
                    Value::Obj(o) => {
                        let o = o.borrow();
                        match &o.kind {
                            ObjKind::Function(_) => Err(RuntimeError {
                                message:
                                    "function call not yet implemented for interpreter"
                                        .into(),
                                stack_trace: vec![],
                            }),
                            _ => Err(RuntimeError {
                                message: "not callable".into(),
                                stack_trace: vec![],
                            }),
                        }
                    }
                    _ => Err(RuntimeError {
                        message: "not callable".into(),
                        stack_trace: vec![],
                    }),
                }
            }
            Expr::Empty => Ok(Value::Nil),
        }
    }

    fn eval_binary(
        &self,
        op: BinOp,
        l: &Expr,
        r: &Expr,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Value, RuntimeError> {
        let lv = self.eval(l, env.clone())?;
        let rv = self.eval(r, env)?;
        match op {
            BinOp::Add => match (&lv, &rv) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Obj(a), Value::Obj(b)) => {
                    let (av, bv) = (a.borrow(), b.borrow());
                    if let (ObjKind::String(s1), ObjKind::String(s2)) = (&av.kind, &bv.kind) {
                        Ok(Value::from_string(format!("{}{}", s1, s2)))
                    } else {
                        Err(RuntimeError {
                            message: "operands must be two numbers or two strings".into(),
                            stack_trace: vec![],
                        })
                    }
                }
                _ => Err(RuntimeError {
                    message: "operands must be two numbers or two strings".into(),
                    stack_trace: vec![],
                }),
            },
            BinOp::Sub => num_bin(lv, rv, |a, b| a - b),
            BinOp::Mul => num_bin(lv, rv, |a, b| a * b),
            BinOp::Div => num_bin(lv, rv, |a, b| a / b),
            BinOp::Eq => Ok(Value::Bool(lv.equals(&rv))),
            BinOp::Ne => Ok(Value::Bool(!lv.equals(&rv))),
            BinOp::Lt => cmp_num(lv, rv, |a, b| a < b),
            BinOp::Le => cmp_num(lv, rv, |a, b| a <= b),
            BinOp::Gt => cmp_num(lv, rv, |a, b| a > b),
            BinOp::Ge => cmp_num(lv, rv, |a, b| a >= b),
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    fn from_literal(&self, lit: Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(n),
            Literal::Str(s) => Value::from_string(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Nil | Literal::None => Value::Nil,
        }
    }
}

fn num_bin<F: Fn(f64, f64) -> f64>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        _ => Err(RuntimeError {
            message: "operands must be numbers".into(),
            stack_trace: vec![],
        }),
    }
}

fn cmp_num<F: Fn(f64, f64) -> bool>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
        _ => Err(RuntimeError {
            message: "operands must be numbers".into(),
            stack_trace: vec![],
        }),
    }
}
