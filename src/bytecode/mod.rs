use std::fmt::Display;

mod chunk;
pub use chunk::*;

mod lines;
pub use lines::*;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Return,
    Constant { index: u8 },
    LongConstant { index: u32 },
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal { name_index: u32 },
    FetchGlobal { name_index: u32 },
    SetGlobal { name_index: u32 },
    SetLocal { index: u32 },
    GetLocal { index: u32 },
    JumpIfFalse { offset: u32 },
    Jump { offset: u32 },
    JumpBack { offset: u32 },
}

impl Instruction {
    pub fn disassemble(&self, f: &mut std::fmt::Formatter<'_>, offset: u32, chunk: &Chunk) -> std::fmt::Result {
        f.write_fmt(format_args!("{offset:4} "))?;

        let line = chunk.line(offset);
        if offset > 0 && line == chunk.line(offset - 1) {
            f.write_str("   | ")?;
        } else {
            f.write_fmt(format_args!("{:4} ", chunk.line(offset)))?;
        }

        match self {
            Instruction::Return => f.write_str("OP_RETURN"),
            Instruction::Constant { index } => f.write_fmt(format_args!("OP_CONSTANT {index} '{}'", chunk.constant(*index as usize))),
            Instruction::LongConstant { index } => f.write_fmt(format_args!("OP_LONG_CONSTANT {index} '{}'", chunk.constant(*index as usize))),
            Instruction::Negate => f.write_str("OP_NEGATE"),
            Instruction::Add => f.write_str("OP_ADD"),
            Instruction::Subtract => f.write_str("OP_SUBTRACT"),
            Instruction::Multiply => f.write_str("OP_MULTIPLY"),
            Instruction::Divide => f.write_str("OP_DIVIDE"),
            Instruction::Not => f.write_str("OP_NOT"),
            Instruction::Equal => f.write_str("OP_EQUAL"),
            Instruction::Greater => f.write_str("OP_GREATER"),
            Instruction::Less => f.write_str("OP_LESS"),
            Instruction::Print => f.write_str("OP_PRINT"),
            Instruction::Pop => f.write_str("OP_POP"),
            Instruction::DefineGlobal { name_index } => f.write_fmt(format_args!("OP_DEFINE_GLOBAL ({})", chunk.constant(*name_index as usize))),
            Instruction::FetchGlobal { name_index } => f.write_fmt(format_args!("OP_FETCH_GLOBAL ({})", chunk.constant(*name_index as usize))),
            Instruction::SetGlobal { name_index } => f.write_fmt(format_args!("OP_SET_GLOBAL ({})", chunk.constant(*name_index as usize))),
            Instruction::SetLocal { index } => f.write_fmt(format_args!("OP_SET_LOCAL ({index})")),
            Instruction::GetLocal { index } => f.write_fmt(format_args!("OP_GET_LOCAL ({index})")),
            Instruction::JumpIfFalse { offset } => f.write_fmt(format_args!("OP_JUMP_IF_FALSE ({offset})")),
            Instruction::Jump { offset } => f.write_fmt(format_args!("OP_JUMP ({offset})")),
            Instruction::JumpBack { offset } => f.write_fmt(format_args!("OP_JUMP_BACK ({offset})")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Double(f64),
    Bool(bool),
    Nil,
    String(String),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Double(_) | Value::String(_) => false,
            Value::Bool(v) => !v,
            Value::Nil => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Double(v) => f.write_fmt(format_args!("{v}")),
            Value::Bool(v) => f.write_fmt(format_args!("{v}")),
            Value::Nil => f.write_fmt(format_args!("nil")),
            Value::String(v) => f.write_fmt(format_args!("{v}")),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Double(l), Value::Double(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}
