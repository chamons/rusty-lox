use std::collections::HashMap;

use thiserror::Error;
use tracing::{debug, trace};

use crate::bytecode::{Chunk, Instruction, Value};

mod function;
pub use function::Function;

#[derive(Debug, Default)]
pub struct VMSettings {
    pub capture_prints: bool,
}

#[derive(Debug)]
pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    settings: VMSettings,
    globals: HashMap<String, Value>,

    // If capture_prints is set then do not print to stdout
    // store here (for integration testing and such)
    pub captured_prints: Vec<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum InterpretErrors {
    #[error("Popped value off stack with no value remaining")]
    PoppedEndOfStack,

    #[error("Invalid runtime type found")]
    InvalidRuntimeType,

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self::new_from_settings(VMSettings::default())
    }

    pub fn new_from_settings(settings: VMSettings) -> Self {
        VM {
            ip: 0,
            stack: vec![],
            globals: HashMap::new(),
            settings,
            captured_prints: vec![],
        }
    }

    pub fn is_stack_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn pop(&mut self) -> Result<Value, InterpretErrors> {
        self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)
    }

    fn peek(&mut self) -> Result<&Value, InterpretErrors> {
        self.stack.last().ok_or(InterpretErrors::PoppedEndOfStack)
    }

    fn pop_double(&mut self) -> Result<f64, InterpretErrors> {
        let value = self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)?;
        match value {
            Value::Double(v) => Ok(v),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }

    fn pop_falsey(&mut self) -> Result<bool, InterpretErrors> {
        Ok(self.pop()?.is_falsey())
    }

    fn peek_falsey(&mut self) -> Result<bool, InterpretErrors> {
        Ok(self.peek()?.is_falsey())
    }

    fn fetch_constant_name(&mut self, chunk: &Chunk, index: usize) -> Result<String, InterpretErrors> {
        match chunk.constant(index) {
            Value::String(name) => Ok(name.clone()),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErrors> {
        loop {
            let Some(instruction) = &chunk.code.get(self.ip) else {
                return Ok(());
            };

            self.ip += 1;

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
                Instruction::DefineGlobal { name_index } => {
                    let name = self.fetch_constant_name(chunk, *name_index as usize)?;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Instruction::FetchGlobal { name_index } => {
                    let name = self.fetch_constant_name(chunk, *name_index as usize)?;
                    match self.globals.get(&name) {
                        Some(value) => {
                            self.stack.push(value.clone());
                        }
                        None => return Err(InterpretErrors::UndefinedVariable(name)),
                    }
                }
                Instruction::SetGlobal { name_index } => {
                    let name = self.fetch_constant_name(chunk, *name_index as usize)?;
                    if !self.globals.contains_key(&name) {
                        return Err(InterpretErrors::UndefinedVariable(name));
                    }
                    let value = self.peek()?.clone();
                    self.globals.insert(name, value);
                }
                Instruction::SetLocal { index } => {
                    let value = self.peek()?.clone();
                    self.stack[*index as usize] = value;
                }
                Instruction::GetLocal { index } => {
                    self.stack.push(self.stack[*index as usize].clone());
                }
                Instruction::JumpIfFalse { offset } => {
                    if self.peek_falsey()? {
                        self.ip += *offset as usize;
                    }
                }
                Instruction::Jump { offset } => {
                    self.ip += *offset as usize;
                }
                Instruction::JumpBack { offset } => {
                    self.ip -= *offset as usize;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        bytecode::{Chunk, Instruction, Value},
        vm::InterpretErrors,
    };

    use super::{VMSettings, VM};

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

    #[test]
    fn globals_write() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::DefineGlobal { name_index }, 123);

        let mut vm = VM::new();
        vm.stack.push(Value::Double(42.0));
        vm.interpret(&chunk).unwrap();
        assert_eq!(vm.globals["asdf"], Value::Double(42.0));
    }

    #[test]
    fn globals_read() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::FetchGlobal { name_index }, 123);

        let mut vm = VM::new();
        vm.globals.insert("asdf".to_string(), Value::Double(42.0));
        vm.interpret(&chunk).unwrap();
        assert_eq!(vm.pop().unwrap(), Value::Double(42.0));
    }

    #[test]
    fn globals_set_not_defined() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::SetGlobal { name_index }, 123);

        let mut vm = VM::new();
        assert_eq!(vm.interpret(&chunk), Err(InterpretErrors::UndefinedVariable("asdf".to_string())));
    }

    #[test]
    fn globals_set_is_defined() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::DefineGlobal { name_index }, 123);
        chunk.write(Instruction::SetGlobal { name_index }, 123);

        let mut vm = VM::new();
        vm.stack.push(Value::Double(12.0));
        vm.stack.push(Value::Double(42.0));
        vm.interpret(&chunk).unwrap();
        assert_eq!(*vm.globals.get(&"asdf".to_string()).unwrap(), Value::Double(12.0));
        assert_eq!(1, vm.stack.len());
    }

    #[test]
    fn locals() {
        let mut chunk = Chunk::new();

        chunk.write(Instruction::SetLocal { index: 0 }, 123);
        chunk.write(Instruction::Pop, 123);
        chunk.write(Instruction::GetLocal { index: 0 }, 123);

        let mut vm = VM::new();
        // The starting value of our local
        vm.stack.push(Value::Double(42.0));
        // The new value
        vm.stack.push(Value::Double(12.0));
        vm.interpret(&chunk).unwrap();

        assert_eq!(2, vm.stack.len());
        assert_eq!(Value::Double(12.0), vm.stack[0]);
        assert_eq!(Value::Double(12.0), vm.stack[1]);
    }

    #[test]
    fn if_jumps() {
        let mut chunk = Chunk::new();

        chunk.write_constant(Value::Bool(false), 123);

        let jump_offset = chunk.write_jump(Instruction::JumpIfFalse { offset: 0 }, 124);
        chunk.write_constant(Value::Nil, 125);
        chunk.write(Instruction::Print, 125);
        chunk.patch_jump(jump_offset).unwrap();
        chunk.write(Instruction::Pop, 124);

        let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });
        vm.interpret(&chunk).unwrap();
        assert!(vm.captured_prints.is_empty());
    }

    #[test]
    fn jumps() {
        let mut chunk = Chunk::new();

        let jump_offset = chunk.write_jump(Instruction::Jump { offset: 0 }, 126);
        chunk.write_constant(Value::Nil, 124);
        chunk.write(Instruction::Print, 124);
        chunk.patch_jump(jump_offset).unwrap();

        let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });
        vm.interpret(&chunk).unwrap();
        assert!(vm.captured_prints.is_empty());
        assert!(vm.stack.is_empty())
    }
}
