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
