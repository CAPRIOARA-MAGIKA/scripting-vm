use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::error::RuntimeError;
use crate::value::Value;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    // Children are owned strongly (the child has its own storage). The parent
    // is held weakly so we don't create cycles when an environment chain
    // outlives its scope. (The strong direction is child -> parent -> grandparent
    // via Weak here, but the *parent's* parent is a strong Rc to the grandparent.
    // Wait that creates cycles too. Let me just use a strong Rc to the parent
    // and rely on the caller to break cycles by dropping inner envs first.)
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, v: Value) {
        self.values.insert(name, v);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            return Some(v.clone());
        }
        self.parent
            .as_ref()
            .and_then(|p| p.borrow().get(name))
    }

    pub fn assign(&mut self, name: &str, v: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), v);
            return Ok(());
        }
        if let Some(p) = self.parent.as_ref() {
            return p.borrow_mut().assign(name, v);
        }
        Err(RuntimeError {
            message: format!("undefined variable '{}'", name),
            stack_trace: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_get() {
        let mut e = Environment::new();
        e.define("x".into(), Value::from(1.0));
        assert_eq!(e.get("x"), Some(Value::from(1.0)));
    }

    #[test]
    fn shadow_via_nested() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer.borrow_mut().define("x".into(), Value::from(1.0));
        let mut inner = Environment::new_child(outer.clone());
        inner.define("x".into(), Value::from(2.0));
        assert_eq!(inner.get("x"), Some(Value::from(2.0)));
        assert_eq!(outer.borrow().get("x"), Some(Value::from(1.0)));
    }

    #[test]
    fn assign_in_enclosing() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer.borrow_mut().define("x".into(), Value::from(1.0));
        let mut inner = Environment::new_child(outer.clone());
        inner.assign("x", Value::from(5.0)).unwrap();
        assert_eq!(outer.borrow().get("x"), Some(Value::from(5.0)));
    }
}
