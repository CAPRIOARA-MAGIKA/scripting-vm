use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::obj::{Obj, ObjKind};

pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Obj(Rc<RefCell<Obj>>),
    Native(NativeFn),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Value {
    pub fn from(n: f64) -> Self {
        Value::Number(n)
    }

    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }

    pub fn from_string(s: String) -> Self {
        Value::Obj(Rc::new(RefCell::new(Obj {
            kind: ObjKind::String(s),
        })))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Obj(a), Value::Obj(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                match (&a.kind, &b.kind) {
                    (ObjKind::String(s1), ObjKind::String(s2)) => s1 == s2,
                    _ => false,
                }
            }
            (Value::Native(a), Value::Native(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Obj(o) => {
                let o = o.borrow();
                match &o.kind {
                    ObjKind::String(s) => write!(f, "{}", s),
                    _ => write!(f, "<object>"),
                }
            }
            Value::Native(_) => write!(f, "<native fn>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_compare_equal() {
        assert!(Value::from(1.0).equals(&Value::from(1.0)));
    }

    #[test]
    fn numbers_display() {
        assert_eq!(Value::from(2.5).to_string(), "2.5");
    }

    #[test]
    fn nil_displays_as_nil() {
        assert_eq!(Value::Nil.to_string(), "nil");
    }
}
