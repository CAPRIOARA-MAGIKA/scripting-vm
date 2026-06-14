use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: usize,
    pub is_open: bool,
    pub value: Value,
}

impl Upvalue {
    pub fn new(index: usize, is_open: bool) -> Self {
        Self {
            index,
            is_open,
            value: Value::Nil,
        }
    }
}

/// Compile-time upvalue reference: either a local slot in the enclosing
/// function, or an upvalue index in the enclosing function's upvalue list.
#[derive(Debug, Clone, Copy)]
pub struct UpvalueRef {
    pub index: u8,
    pub is_local: bool,
}
