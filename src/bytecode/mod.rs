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
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Double(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Double(v) => f.write_fmt(format_args!("{v}")),
        }
    }
}
