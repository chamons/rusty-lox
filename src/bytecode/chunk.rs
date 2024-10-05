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

    pub fn write_constant(&mut self, value: Value, line: u32) {
        self.constants.push(value);
        let index = self.constants.len() - 1;

        if index > u8::MAX as usize {
            self.write(Instruction::LongConstant { index: index as u32 }, line);
        } else {
            self.write(Instruction::Constant { index: index as u8 }, line);
        }
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index as usize]
    }

    pub fn line(&self, index: u32) -> u32 {
        self.lines.get(index).expect("Unknown line for index {index}")
    }

    pub fn code(&self) -> &[Instruction] {
        &self.code
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
    use crate::bytecode::{Instruction, Value};

    use super::Chunk;

    #[test]
    fn disassemble_chunk() {
        let mut chunk = Chunk::new();
        chunk.write(Instruction::Constant { index: 0 }, 123);
        chunk.write(Instruction::LongConstant { index: 1 }, 1230);
        chunk.write(Instruction::Return, 123);
        chunk.constants.push(Value::Double(1.2));
        chunk.constants.push(Value::Double(12.2));

        let output = chunk.to_string();
        // println!("{output}");

        const EXPECTED: &str = "   0  123 OP_CONSTANT 0 '1.2'
   1 1230 OP_LONG_CONSTANT 1 '12.2'
   2  123 OP_RETURN
";
        assert_eq!(output, EXPECTED);
    }

    #[test]
    fn write_constant() {
        let mut chunk = Chunk::new();
        for i in 0..260 {
            chunk.write_constant(Value::Double(i as f64), 123);
        }
        assert!(matches!(chunk.code[255], Instruction::Constant { .. }));
        assert!(matches!(chunk.code[256], Instruction::LongConstant { .. }));
    }
}
