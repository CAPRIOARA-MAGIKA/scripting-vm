use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetUpvalue,
    SetUpvalue,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Closure,
    CloseUpvalue,
    Return,
    // Reserved for v2 — declared so the enum is stable, never emitted by the v1 compiler.
    Class,
    Inherit,
    Method,
    GetProperty,
    SetProperty,
    Invoke,
    SuperInvoke,
}

impl OpCode {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(OpCode::Constant),
            1 => Some(OpCode::Nil),
            2 => Some(OpCode::True),
            3 => Some(OpCode::False),
            4 => Some(OpCode::Pop),
            5 => Some(OpCode::GetLocal),
            6 => Some(OpCode::SetLocal),
            7 => Some(OpCode::GetGlobal),
            8 => Some(OpCode::DefineGlobal),
            9 => Some(OpCode::SetGlobal),
            10 => Some(OpCode::GetUpvalue),
            11 => Some(OpCode::SetUpvalue),
            12 => Some(OpCode::Add),
            13 => Some(OpCode::Sub),
            14 => Some(OpCode::Mul),
            15 => Some(OpCode::Div),
            16 => Some(OpCode::Neg),
            17 => Some(OpCode::Not),
            18 => Some(OpCode::Equal),
            19 => Some(OpCode::Greater),
            20 => Some(OpCode::Less),
            21 => Some(OpCode::Print),
            22 => Some(OpCode::Jump),
            23 => Some(OpCode::JumpIfFalse),
            24 => Some(OpCode::Loop),
            25 => Some(OpCode::Call),
            26 => Some(OpCode::Closure),
            27 => Some(OpCode::CloseUpvalue),
            28 => Some(OpCode::Return),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op as u8);
        self.lines.push(line);
    }

    pub fn write_byte(&mut self, b: u8, line: usize) {
        self.code.push(b);
        self.lines.push(line);
    }

    pub fn write_u16(&mut self, n: u16, line: usize) {
        self.code.push((n >> 8) as u8);
        self.lines.push(line);
        self.code.push((n & 0xff) as u8);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, v: Value) -> Result<u8, crate::error::CompileError> {
        if self.constants.len() >= u8::MAX as usize {
            return Err(crate::error::CompileError {
                line: 0,
                message: "too many constants in one chunk".into(),
            });
        }
        let idx = self.constants.len() as u8;
        self.constants.push(v);
        Ok(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_writes_and_reads() {
        let mut c = Chunk::new();
        let k = c.add_constant(crate::value::Value::Number(7.0)).unwrap();
        c.write(OpCode::Constant, 1);
        c.write_byte(k, 1);
        c.write(OpCode::Return, 1);
        assert_eq!(c.code[0], OpCode::Constant as u8);
        assert_eq!(c.code[1], 0); // constant index
        assert_eq!(c.code[2], OpCode::Return as u8);
    }

    #[test]
    fn opcodes_distinct_u8() {
        let ops = [OpCode::Constant, OpCode::Add, OpCode::Return];
        let bytes: Vec<u8> = ops.iter().map(|o| *o as u8).collect();
        assert!(bytes[0] != bytes[1] && bytes[1] != bytes[2]);
    }
}
