use std::collections::HashMap;

use thiserror::Error;
use tracing::{debug, trace};

use crate::bytecode::{Instruction, Value};

mod frame;
pub use frame::Frame;
mod function;
pub use function::Function;

#[derive(Debug, Default)]
pub struct VMSettings {
    pub capture_prints: bool,
}

#[derive(Debug)]
pub struct VM {
    settings: VMSettings,
    globals: HashMap<String, Value>,
    stack: Vec<Value>,

    // If capture_prints is set then do not print to stdout
    // store here (for integration testing and such)
    pub captured_prints: Vec<String>,

    frames: Vec<Frame>,
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
            frames: vec![],
            stack: vec![],
            globals: HashMap::new(),
            settings,
            captured_prints: vec![],
        }
    }

    pub fn pop(&mut self) -> Result<Value, InterpretErrors> {
        self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn peek(&mut self) -> Result<&Value, InterpretErrors> {
        self.stack.last().ok_or(InterpretErrors::PoppedEndOfStack)
    }

    pub fn pop_double(&mut self) -> Result<f64, InterpretErrors> {
        let value = self.stack.pop().ok_or(InterpretErrors::PoppedEndOfStack)?;
        match value {
            Value::Double(v) => Ok(v),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }

    pub fn pop_falsey(&mut self) -> Result<bool, InterpretErrors> {
        Ok(self.pop()?.is_falsey())
    }

    pub fn peek_falsey(&mut self) -> Result<bool, InterpretErrors> {
        Ok(self.peek()?.is_falsey())
    }

    pub fn is_stack_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn interpret(&mut self, function: Function) -> Result<(), InterpretErrors> {
        self.interpret_frame(Frame::new(function))
    }

    fn current_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    fn move_to_next_instruction(&mut self) -> Option<Instruction> {
        self.current_frame_mut()?.next_instruction()
    }

    pub fn constant(&self, index: usize) -> Value {
        // This unwrap is safe as we do not process constants without having an instruction
        // and thus a frame
        self.current_frame().unwrap().constant(index)
    }

    pub fn fetch_constant_name(&self, index: usize) -> Result<String, InterpretErrors> {
        // This unwrap is safe as we do not process constants without having an instruction
        // and thus a frame
        self.current_frame().unwrap().fetch_constant_name(index)
    }

    fn current_ip_mut(&mut self) -> &mut usize {
        // This unwrap is safe as we do not process constants without having an instruction
        // and thus a frame
        &mut self.current_frame_mut().unwrap().ip
    }

    fn interpret_frame(&mut self, starting_frame: Frame) -> Result<(), InterpretErrors> {
        self.frames.push(starting_frame);

        loop {
            let Some(instruction) = self.move_to_next_instruction() else {
                return Ok(());
            };

            trace!(?instruction, frame = ?self.current_frame(), "Interpreting");

            match instruction {
                Instruction::Return => {}
                Instruction::Constant { index } => {
                    let constant = self.constant(index as usize);
                    debug!(value = %constant, "Interpreted constant");

                    self.push(constant);
                }
                Instruction::LongConstant { index } => {
                    let constant = self.constant(index as usize);
                    debug!(value = %constant, "Interpreted constant");

                    self.push(constant);
                }
                Instruction::Negate => {
                    let v = self.pop_double()?;
                    self.push(Value::Double(-v));
                }
                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Double(a), Value::Double(b)) => {
                            self.push(Value::Double(a + b));
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::String(a + &b));
                        }
                        _ => return Err(InterpretErrors::InvalidRuntimeType),
                    }
                }
                Instruction::Subtract => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.push(Value::Double(a - b));
                }
                Instruction::Multiply => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.push(Value::Double(a * b));
                }
                Instruction::Divide => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.push(Value::Double(a / b));
                }
                Instruction::Not => {
                    let a = self.pop_falsey()?;
                    self.push(Value::Bool(a));
                }
                Instruction::Equal => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(Value::Bool(a == b));
                }
                Instruction::Greater => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.push(Value::Bool(a > b));
                }
                Instruction::Less => {
                    let b = self.pop_double()?;
                    let a = self.pop_double()?;
                    self.push(Value::Bool(a < b));
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
                    let name = self.fetch_constant_name(name_index as usize)?;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Instruction::FetchGlobal { name_index } => {
                    let name = self.fetch_constant_name(name_index as usize)?;
                    match self.globals.get(&name) {
                        Some(value) => {
                            self.push(value.clone());
                        }
                        None => return Err(InterpretErrors::UndefinedVariable(name)),
                    }
                }
                Instruction::SetGlobal { name_index } => {
                    let name = self.fetch_constant_name(name_index as usize)?;
                    if !self.globals.contains_key(&name) {
                        return Err(InterpretErrors::UndefinedVariable(name));
                    }
                    let value = self.peek()?.clone();
                    self.globals.insert(name, value);
                }
                Instruction::SetLocal { index } => {
                    let value = self.peek()?.clone();
                    self.stack[index as usize] = value;
                }
                Instruction::GetLocal { index } => {
                    self.stack.push(self.stack[index as usize].clone());
                }
                Instruction::JumpIfFalse { offset } => {
                    if self.peek_falsey()? {
                        *self.current_ip_mut() += offset as usize;
                    }
                }
                Instruction::Jump { offset } => {
                    *self.current_ip_mut() += offset as usize;
                }
                Instruction::JumpBack { offset } => {
                    *self.current_ip_mut() -= offset as usize;
                }
                Instruction::Call { arg_count } => {
                    todo!()
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
        vm::{Frame, InterpretErrors},
    };

    use super::{Function, VMSettings, VM};

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
    fn executes_return_zero() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Return, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        vm.interpret(function).unwrap();
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

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        vm.interpret(function).unwrap();
        assert_eq!(vm.stack[0], Value::Double(-0.8214285714285714));
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn negate_boolean(#[case] input: bool) {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(input), 123);
        chunk.write(Instruction::Not, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        vm.interpret(function).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(!input));
    }

    #[test]
    fn negate_nil() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Nil, 123);
        chunk.write(Instruction::Not, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        vm.interpret(function).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(true));
    }

    #[test]
    fn add_wrong_types() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(true), 123);
        chunk.write_constant(Value::Double(1.2), 123);
        chunk.write(Instruction::Add, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        assert!(vm.interpret(function).is_err());
    }

    #[test]
    fn new_constants() {
        let mut chunk = Chunk::new();
        chunk.write_constant(Value::Bool(true), 123);
        chunk.write_constant(Value::Nil, 123);
        chunk.write(Instruction::Return, 123);
    }

    #[test]
    fn globals_write() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::DefineGlobal { name_index }, 123);

        let function = Function::new_script(chunk);
        let frame = Frame::new(function);

        let mut vm = VM::new();
        vm.stack.push(Value::Double(42.0));

        vm.interpret_frame(frame).unwrap();
        assert_eq!(vm.globals["asdf"], Value::Double(42.0));
    }

    #[test]
    fn globals_read() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::FetchGlobal { name_index }, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        vm.globals.insert("asdf".to_string(), Value::Double(42.0));
        vm.interpret(function).unwrap();
        assert_eq!(vm.pop().unwrap(), Value::Double(42.0));
    }

    #[test]
    fn globals_set_not_defined() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::SetGlobal { name_index }, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();
        assert_eq!(vm.interpret(function), Err(InterpretErrors::UndefinedVariable("asdf".to_string())));
    }

    #[test]
    fn globals_set_is_defined() {
        let mut chunk = Chunk::new();

        let name_index = chunk.make_constant(Value::String("asdf".to_string()));
        chunk.write(Instruction::DefineGlobal { name_index }, 123);
        chunk.write(Instruction::SetGlobal { name_index }, 123);

        let function = Function::new_script(chunk);
        let frame = Frame::new(function);

        let mut vm = VM::new();
        vm.stack.push(Value::Double(12.0));
        vm.stack.push(Value::Double(42.0));

        vm.interpret_frame(frame).unwrap();
        assert_eq!(*vm.globals.get(&"asdf".to_string()).unwrap(), Value::Double(12.0));
        assert_eq!(1, vm.stack.len());
    }

    #[test]
    fn locals() {
        let mut chunk = Chunk::new();

        chunk.write(Instruction::SetLocal { index: 0 }, 123);
        chunk.write(Instruction::Pop, 123);
        chunk.write(Instruction::GetLocal { index: 0 }, 123);

        let function = Function::new_script(chunk);

        let frame = Frame::new(function);

        let mut vm = VM::new();
        // The starting value of our local
        vm.stack.push(Value::Double(42.0));
        // The new value
        vm.stack.push(Value::Double(12.0));

        vm.interpret_frame(frame).unwrap();

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

        let function = Function::new_script(chunk);

        let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });
        vm.interpret(function).unwrap();
        assert!(vm.captured_prints.is_empty());
    }

    #[test]
    fn jumps() {
        let mut chunk = Chunk::new();

        let jump_offset = chunk.write_jump(Instruction::Jump { offset: 0 }, 126);
        chunk.write_constant(Value::Nil, 124);
        chunk.write(Instruction::Print, 124);
        chunk.patch_jump(jump_offset).unwrap();

        let function = Function::new_script(chunk);

        let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });
        vm.interpret(function).unwrap();
        assert!(vm.captured_prints.is_empty());
        assert!(vm.is_stack_empty())
    }
}
