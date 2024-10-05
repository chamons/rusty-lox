use std::fmt::Display;

use super::{Instruction, Lines, Value};

#[derive(Debug, Default)]
pub struct Chunk {
    code: Vec<Instruction>,
    constants: Vec<Value>,
    lines: Lines,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, instruction: Instruction, line: u32) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub fn constant(&self, index: u8) -> &Value {
        &self.constants[index as usize]
    }

    pub fn line(&self, index: u32) -> u32 {
        self.lines.get(index).expect("Unknown line for index {index}")
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (offset, instruction) in self.code.iter().enumerate() {
            instruction.disassemble(f, offset as u32, self)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::{self, Value};

    use super::Chunk;

    #[test]
    fn disassemble_chunk() {
        let mut chunk = Chunk::new();
        chunk.write(bytecode::Instruction::Constant { index: 0 }, 123);
        chunk.write(bytecode::Instruction::Return, 123);
        chunk.constants.push(Value::Double(1.2));

        let output = chunk.to_string();
        const EXPECTED: &str = "   0  123 OP_CONSTANT 0 '1.2'
   1    | OP_RETURN
";
        assert_eq!(output, EXPECTED);
    }
}
