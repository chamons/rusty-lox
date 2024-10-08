use thiserror::Error;
use tracing::{debug, trace};

use crate::bytecode::{Chunk, Instruction, Value};

#[derive(Debug, Default)]
pub struct VMSettings {
    pub capture_prints: bool,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>,
    settings: VMSettings,

    // If capture_prints is set then do not print to stdout
    // store here (for integration testing and such)
    pub captured_prints: Vec<String>,
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
    pub fn new() -> Self {
        Self::new_from_settings(VMSettings::default())
    }

    pub fn new_from_settings(settings: VMSettings) -> Self {
        VM {
            stack: vec![],
            settings,
            captured_prints: vec![],
        }
    }

    fn pop(&mut self) -> Result<Value, InterpretErrors> {
        self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)
    }

    fn pop_double(&mut self) -> Result<f64, InterpretErrors> {
        let value = self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)?;
        match value {
            Value::Double(v) => Ok(v),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }

    fn pop_falsey(&mut self) -> Result<bool, InterpretErrors> {
        let value = self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)?;
        Ok(match value {
            Value::Double(_) | Value::String(_) => false,
            Value::Bool(v) => !v,
            Value::Nil => true,
        })
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErrors> {
        for instruction in chunk.code() {
            trace!(?instruction, stack = ?self.stack, "Interpreting");

            match instruction {
                Instruction::Return => {}
                Instruction::Constant { index } => {
                    let constant = chunk.constant(*index as usize);
                    self.stack.push(constant.clone());
                    debug!(value = %constant, "Interpreted constant");
                }
                Instruction::LongConstant { index } => {
                    let constant = chunk.constant(*index as usize);
                    self.stack.push(constant.clone());
                    debug!(value = %constant, "Interpreted constant");
                }
                Instruction::Negate => {
                    let v = self.pop_double()?;
                    self.stack.push(Value::Double(-v));
                }
                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Double(a), Value::Double(b)) => {
                            self.stack.push(Value::Double(a + b));
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.stack.push(Value::String(a + &b));
                        }
                        _ => return Err(InterpretErrors::InvalidRuntimeType),
                    }
                }
                Instruction::Subtract => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a - b));
                }
                Instruction::Multiply => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a * b));
                }
                Instruction::Divide => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Double(a / b));
                }
                Instruction::Not => {
                    let a = self.pop_falsey()?;
                    self.stack.push(Value::Bool(a));
                }
                Instruction::Equal => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.stack.push(Value::Bool(a == b));
                }
                Instruction::Greater => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Bool(a > b));
                }
                Instruction::Less => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.stack.push(Value::Bool(a < b));
                }
                Instruction::Print => {
                    let a = self.pop()?;
                    if self.settings.capture_prints {
                        self.captured_prints.push(format!("{a}"));
                    } else {
                        println!("{a}");
                    }
                }
                Instruction::Pop => {
                    let _ = self.pop()?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::bytecode::{Chunk, Instruction, Value};

    use super::VM;

    #[test]
    fn executes_return_zero() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Return, 123);

        let mut vm = VM::new();
        vm.interpret(&chunk).unwrap();
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

        let mut vm = VM::new();
        vm.interpret(&chunk).unwrap();
        assert_eq!(vm.stack[0], Value::Double(-0.8214285714285714));
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn negate_boolean(#[case] input: bool) {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(input), 123);
        chunk.write(Instruction::Not, 123);

        let mut vm = VM::new();
        vm.interpret(&chunk).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(!input));
    }

    #[test]
    fn negate_nil() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Nil, 123);
        chunk.write(Instruction::Not, 123);

        let mut vm = VM::new();
        vm.interpret(&chunk).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(true));
    }

    #[test]
    fn add_wrong_types() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(true), 123);
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Add, 123);

        let mut vm = VM::new();
        assert!(vm.interpret(&chunk).is_err());
    }

    #[test]
    fn new_constants() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(true), 123);
        chunk.write_constant(Value::Nil, 123);
        chunk.write(Instruction::Return, 123);
    }

    #[test]
    fn falsey() {
        let mut vm = VM::new();
        vm.stack.push(Value::Double(1.2));
        vm.stack.push(Value::Double(0.0));
        vm.stack.push(Value::Nil);
        vm.stack.push(Value::Bool(true));
        vm.stack.push(Value::Bool(false));
        assert!(vm.pop_falsey().unwrap());
        assert!(!vm.pop_falsey().unwrap());
        assert!(vm.pop_falsey().unwrap());
        assert!(!vm.pop_falsey().unwrap());
        assert!(!vm.pop_falsey().unwrap());
    }
}
