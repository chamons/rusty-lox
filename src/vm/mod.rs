use thiserror::Error;

use crate::bytecode::Chunk;

#[derive(Debug, Default)]
pub struct VM {}

#[derive(Error, Debug)]
pub enum InterpretErrors {
    #[error("Reached the end of a chunk unexpectedly")]
    ReachedEndOfChunk,
}

impl VM {
    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErrors> {
        for instruction in chunk.code() {
            match instruction {
                crate::bytecode::Instruction::Return => return Ok(()),
                crate::bytecode::Instruction::Constant { index } => {
                    let constant = chunk.constant(*index as usize);
                    println!("{constant}");
                }
                crate::bytecode::Instruction::LongConstant { index } => {
                    let constant = chunk.constant(*index as usize);
                    println!("{constant}");
                }
            }
        }

        Err(InterpretErrors::ReachedEndOfChunk)
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::{Chunk, Instruction, Value};

    use super::VM;

    #[test]
    fn executes_return_zero() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Return, 123);

        let mut vm = VM::default();
        vm.interpret(&chunk).unwrap();
    }
}
