use std::fmt::Debug;

mod frontend;
pub use frontend::*;

mod chunks;
pub use chunks::*;

mod vm;
pub use vm::*;

mod compiler;
pub use compiler::*;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum OpCode {
    Return,
    Constant(usize),
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Greater,
    Less,
}

impl OpCode {
    pub fn disassemble(&self, chunk: &Chunk) -> String {
        match self {
            OpCode::Return => "OP_RETURN".to_string(),
            OpCode::Constant(index) => format!("OP_CONSTANT\t{} {:?}", index, chunk.values[*index]),
            OpCode::Negate => "OP_NEGATE".to_string(),
            OpCode::Add => "OP_ADD".to_string(),
            OpCode::Subtract => "OP_SUBTRACT".to_string(),
            OpCode::Multiply => "OP_MULTIPLY".to_string(),
            OpCode::Divide => "OP_DIVIDE".to_string(),
            OpCode::Not => "OP_NOT".to_string(),
            OpCode::Equal => "OP_EQUAL".to_string(),
            OpCode::Greater => "OP_GREATER".to_string(),
            OpCode::Less => "OP_LESS".to_string(),
        }
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum OpValue {
    Double(f64),
    Boolean(bool),
    Nil,
}

impl OpValue {
    fn as_double(&self) -> Option<f64> {
        match self {
            OpValue::Double(v) => Some(*v),
            _ => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            OpValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    fn as_nil(&self) -> Option<()> {
        match self {
            OpValue::Nil => Some(()),
            _ => None,
        }
    }
}

impl Debug for OpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Double(v) => write!(f, "'{}'", v),
            Self::Boolean(v) => write!(f, "'{}'", v),
            Self::Nil => write!(f, "nil"),
        }
    }
}
