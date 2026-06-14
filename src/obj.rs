use crate::opcode::Chunk;
use crate::upvalue::{Upvalue, UpvalueRef};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Obj {
    pub kind: ObjKind,
}

#[derive(Clone, Debug)]
pub enum ObjKind {
    String(String),
    Function(FunctionObj),
    Closure(ClosureObj),
    Upvalue(Upvalue),
}

#[derive(Clone, Debug)]
pub struct FunctionObj {
    pub name: String,
    pub arity: u8,
    pub chunk: Option<Chunk>,
    pub upvalues: Vec<UpvalueRef>,
}

#[derive(Clone, Debug)]
pub struct ClosureObj {
    pub function: Box<FunctionObj>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}
