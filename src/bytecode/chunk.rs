use std::fmt::Display;

use super::{Instruction, Lines, Value};

#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<Instruction>,
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

    pub fn make_constant(&mut self, value: Value) -> u32 {
        if let Some(existing_index) = self.constants.iter().position(|c| *c == value) {
            existing_index as u32
        } else {
            self.constants.push(value);
            (self.constants.len() - 1) as u32
        }
    }

    pub fn write_constant(&mut self, value: Value, line: u32) {
        let index = self.make_constant(value);

        if index > u8::MAX as u32 {
            self.write(Instruction::LongConstant { index }, line);
        } else {
            self.write(Instruction::Constant { index: index as u8 }, line);
        }
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    pub fn line(&self, index: u32) -> u32 {
        self.lines.get(index).expect("Unknown line for index {index}")
    }

    pub fn code(&self) -> &[Instruction] {
        &self.code
    }

    pub fn write_jump(&mut self, instruction: Instruction, line: u32) -> usize {
        self.write(instruction, line);
        self.code.len() - 1
    }

    pub fn patch_jump(&mut self, jump_offset: usize) -> eyre::Result<()> {
        let new_offset = self.code.len() - jump_offset - 1;

        let instruction = &mut self.code[jump_offset];
        match instruction {
            Instruction::JumpIfFalse { offset } => {
                *offset = new_offset as u32;
                Ok(())
            }
            Instruction::Jump { offset } => {
                *offset = new_offset as u32;
                Ok(())
            }
            i => Err(eyre::eyre!("Invalid instruction {i:?} found when trying to patch a jump")),
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\n")?;
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
        chunk.write(Instruction::LongConstant { index: 1 }, 124);
        chunk.constants.push(Value::Double(1.2));
        chunk.constants.push(Value::Double(12.2));

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::Add, 125);
        chunk.write_constant(Value::Double(1.0), 125);
        chunk.write_constant(Value::Double(3.0), 125);
        chunk.write(Instruction::DefineGlobal { name_index }, 125);
        chunk.write(Instruction::Return, 126);

        let output = chunk.to_string();
        // println!("{output}");

        const EXPECTED: &str = "
   0  123 OP_CONSTANT 0 '1.2'
   1  124 OP_LONG_CONSTANT 1 '12.2'
   2  125 OP_ADD
   3    | OP_CONSTANT 3 '1'
   4    | OP_CONSTANT 4 '3'
   5    | OP_DEFINE_GLOBAL (asdf)
   6  126 OP_RETURN
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

    #[test]
    fn write_jump() {
        let mut chunk = Chunk::new();

        chunk.write(Instruction::Constant { index: 0 }, 123);
        chunk.write(Instruction::LongConstant { index: 1 }, 124);
        chunk.constants.push(Value::Double(1.2));
        chunk.constants.push(Value::Double(12.2));
        chunk.write(Instruction::Add, 125);

        let offset = chunk.write_jump(Instruction::JumpIfFalse { offset: 0 }, 126);
        assert!(matches!(chunk.code[offset], Instruction::JumpIfFalse { .. }));

        chunk.write(Instruction::Constant { index: 0 }, 123);
        chunk.write(Instruction::LongConstant { index: 1 }, 124);
        chunk.constants.push(Value::Double(1.2));
        chunk.constants.push(Value::Double(12.2));
        chunk.write(Instruction::Add, 125);
        chunk.write(Instruction::Pop, 125);
        chunk.patch_jump(offset).unwrap();

        if let Instruction::JumpIfFalse { offset } = chunk.code[offset] {
            assert_eq!(offset, 4);
        }
    }
}
