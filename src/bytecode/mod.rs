mod chunk;
use std::fmt::{Display, Write};

pub use chunk::*;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Return,
    Constant { index: u8 },
}

impl Instruction {
    pub fn disassemble(&self, f: &mut std::fmt::Formatter<'_>, offset: usize, chunk: &Chunk) -> std::fmt::Result {
        f.write_fmt(format_args!("{offset:4} "))?;

        let line = chunk.line(offset);
        if offset > 0 && line == chunk.line(offset - 1) {
            f.write_str("   | ")?;
        } else {
            f.write_fmt(format_args!("{:4} ", chunk.line(offset)))?;
        }

        match self {
            Instruction::Return => f.write_str("OP_RETURN"),
            Instruction::Constant { index } => f.write_fmt(format_args!("OP_CONSTANT {index} '{}'", chunk.constant(*index))),
        }
    }
}

#[derive(Debug, PartialEq)]
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
