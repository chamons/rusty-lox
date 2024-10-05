use thiserror::Error;
use tracing::{debug, trace};

use crate::bytecode::{Chunk, Value};

#[derive(Debug, Default)]
pub struct VM {
    stack: Vec<Value>,
}

#[derive(Error, Debug)]
pub enum InterpretErrors {
    #[error("Reached the end of a chunk unexpectedly")]
    ReachedEndOfChunk,

    #[error("Popped value off stack with no value remaining")]
    PoppedEndOfStack,

    #[error("Invalid runtime type found")]
    InvalidRuntimeType,
}

impl VM {
    fn pop_double(&mut self) -> Result<f64, InterpretErrors> {
        let value = self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)?;
        match value {
            Value::Double(v) => Ok(v),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }

    pub fn interpret(&mut self, source: &String) -> Result<(), InterpretErrors> {
        Ok(())
    }

    pub fn interpret_chunk(&mut self, chunk: &Chunk) -> Result<(), InterpretErrors> {
        for instruction in chunk.code() {
            trace!(?instruction, stack = ?self.stack, "Interpreting");

            match instruction {
                crate::bytecode::Instruction::Return => return Ok(()),
                crate::bytecode::Instruction::Constant { index } => {
                    let constant = chunk.constant(*index as usize);
                    self.stack.push(constant.clone());
                    debug!(value = %constant, "Interpreted constant");
                }
                crate::bytecode::Instruction::LongConstant { index } => {
                    let constant = chunk.constant(*index as usize);
                    self.stack.push(constant.clone());
                    debug!(value = %constant, "Interpreted constant");
                }
                crate::bytecode::Instruction::Negate => {
                    let v = self.pop_double()?;
                    self.stack.push(Value::Double(-v));
                }
                crate::bytecode::Instruction::Add => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a + b));
                }
                crate::bytecode::Instruction::Subtract => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a - b));
                }
                crate::bytecode::Instruction::Multiply => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a * b));
                }
                crate::bytecode::Instruction::Divide => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a / b));
                }
            }
        }

        Err(InterpretErrors::ReachedEndOfChunk)
    }
}

#[cfg(test)]
mod tests {
    use tracing::level_filters::LevelFilter;

    use crate::bytecode::{Chunk, Instruction, Value};

    use super::VM;

    #[test]
    fn executes_return_zero() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Return, 123);

        let mut vm = VM::default();
        vm.interpret_chunk(&chunk).unwrap();
    }

    #[test]
    fn basic_math() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write_constant(Value::Double(3.4), 123);
        chunk.write(Instruction::Add, 123);
        chunk.write_constant(Value::Double(5.6), 123);
        chunk.write(Instruction::Divide, 123);
        chunk.write(Instruction::Negate, 123);
        chunk.write(Instruction::Return, 125);

        let mut vm = VM::default();
        vm.interpret_chunk(&chunk).unwrap();
        assert_eq!(vm.stack[0], Value::Double(-0.8214285714285714));
    }
}
