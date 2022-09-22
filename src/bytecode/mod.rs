use std::fmt::Debug;

mod frontend;
pub use frontend::*;

mod chunks;
pub use chunks::*;

mod vm;
pub use vm::*;

mod compiler;
pub use compiler::*;

mod intern;
pub use intern::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
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
    Print,
    Pop,
}

impl OpCode {
    pub fn disassemble(&self, chunk: &Chunk, strings: &Interner) -> String {
        match self {
            OpCode::Return => "OP_RETURN".to_string(),
            OpCode::Constant(index) => {
                format!("OP_CONSTANT\t{} {:?}", index, chunk.values[*index].print(strings))
            }
            OpCode::Negate => "OP_NEGATE".to_string(),
            OpCode::Add => "OP_ADD".to_string(),
            OpCode::Subtract => "OP_SUBTRACT".to_string(),
            OpCode::Multiply => "OP_MULTIPLY".to_string(),
            OpCode::Divide => "OP_DIVIDE".to_string(),
            OpCode::Not => "OP_NOT".to_string(),
            OpCode::Equal => "OP_EQUAL".to_string(),
            OpCode::Greater => "OP_GREATER".to_string(),
            OpCode::Less => "OP_LESS".to_string(),
            OpCode::Print => "OP_PRINT".to_string(),
            OpCode::Pop => "OP_POP".to_string(),
        }
    }
}

#[derive(PartialEq, PartialOrd, Clone)]
pub enum OpValue {
    Double(f64),
    Boolean(bool),
    Nil,
    Object(ObjectType),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub enum ObjectType {
    String(InternedString),
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

    fn as_string(&self) -> Option<InternedString> {
        match self {
            OpValue::Object(o) => match o {
                ObjectType::String(v) => Some(*v),
            },
            _ => None,
        }
    }

    pub fn print(&self, strings: &Interner) -> String {
        match self {
            OpValue::Double(v) => v.to_string(),
            OpValue::Boolean(v) => v.to_string(),
            OpValue::Nil => "nil".to_string(),
            OpValue::Object(v) => match v {
                ObjectType::String(interned) => strings.lookup(*interned).to_string(),
            },
        }
    }
}

impl Debug for OpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Double(v) => write!(f, "'{}'", v),
            Self::Boolean(v) => write!(f, "'{}'", v),
            Self::Object(v) => write!(f, "'{:?}'", v),
            Self::Nil => write!(f, "nil"),
        }
    }
}
