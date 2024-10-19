use std::{collections::HashMap, sync::Arc};

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
    pub skip_error_stacktrace: bool,
}

impl VMSettings {
    pub fn test_default() -> Self {
        VMSettings {
            capture_prints: true,
            skip_error_stacktrace: true,
        }
    }
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

    #[error("Incorrect number of arguments (expected {0}, received {1})")]
    IncorrectArgumentCount(u32, u32),
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

    pub fn peek(&self) -> Result<&Value, InterpretErrors> {
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

    pub fn peek_falsey(&self) -> Result<bool, InterpretErrors> {
        Ok(self.peek()?.is_falsey())
    }

    pub fn is_stack_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn interpret(&mut self, function: Function) -> Result<(), InterpretErrors> {
        let function = Arc::new(function);
        match self.interpret_frame(Frame::new(function.clone())) {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("{err}");
                for frame in self.frames.iter().rev() {
                    let location = function.name.as_deref().unwrap_or("script");
                    println!("[line {}] in {location}", frame.function.chunk.line(frame.ip as u32 - 1));
                }

                Err(err)
            }
        }
    }

    fn interpret_frame(&mut self, starting_frame: Frame) -> Result<(), InterpretErrors> {
        self.frames.push(starting_frame);

        loop {
            let Some(current_frame) = self.frames.last_mut() else {
                return Ok(());
            };

            let Some(instruction) = current_frame.next_instruction().clone() else {
                return Ok(());
            };

            trace!(?instruction, frame = ?current_frame, "Interpreting");

            match instruction {
                Instruction::Return => {}
                Instruction::Constant { index } => {
                    let constant = current_frame.constant(index as usize);
                    debug!(value = %constant, "Interpreted constant");

                    self.push(constant);
                }
                Instruction::LongConstant { index } => {
                    let constant = current_frame.constant(index as usize);
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
                    let name = current_frame.fetch_constant_name(name_index as usize)?;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Instruction::FetchGlobal { name_index } => {
                    let name = current_frame.fetch_constant_name(name_index as usize)?;
                    match self.globals.get(&name) {
                        Some(value) => {
                            self.push(value.clone());
                        }
                        None => return Err(InterpretErrors::UndefinedVariable(name)),
                    }
                }
                Instruction::SetGlobal { name_index } => {
                    let name = current_frame.fetch_constant_name(name_index as usize)?;
                    if !self.globals.contains_key(&name) {
                        return Err(InterpretErrors::UndefinedVariable(name));
                    }
                    let value = self.peek()?.clone();
                    self.globals.insert(name, value);
                }
                Instruction::SetLocal { index } => {
                    let frame_stack_offset = current_frame.stack_offset;
                    let value = self.peek()?.clone();
                    self.stack[frame_stack_offset + index as usize] = value;
                }
                Instruction::GetLocal { index } => {
                    let frame_stack_offset = current_frame.stack_offset;
                    let value = self.stack[frame_stack_offset + index as usize].clone();
                    self.stack.push(value);
                }
                Instruction::JumpIfFalse { offset } => {
                    if self.peek_falsey()? {
                        // We can not use current_frame as we have to borrow
                        // self and would get double borrow
                        // so refetch current frame
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                Instruction::Jump { offset } => {
                    current_frame.ip += offset as usize;
                }
                Instruction::JumpBack { offset } => {
                    current_frame.ip -= offset as usize;
                }
                Instruction::Call { arg_count } => {
                    let function = self
                        .stack
                        .get(self.stack.len() - arg_count as usize - 1)
                        .ok_or(InterpretErrors::PoppedEndOfStack)?;

                    let function = match function {
                        Value::Function(v) => Ok(v),
                        _ => Err(InterpretErrors::InvalidRuntimeType),
                    }?;

                    if function.arity != arg_count {
                        return Err(InterpretErrors::IncorrectArgumentCount(function.arity, arg_count));
                    }

                    self.frames.push(Frame {
                        function: function.clone(),
                        ip: 0,
                        stack_offset: arg_count as usize,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
        let frame = Frame::new(Arc::new(function));

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
        let frame = Frame::new(Arc::new(function));

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

        let frame = Frame::new(Arc::new(function));

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
    fn locals_nested_frames() {
        let mut chunk = Chunk::new();

        chunk.write(Instruction::SetLocal { index: 0 }, 123);
        chunk.write(Instruction::Pop, 123);
        chunk.write(Instruction::GetLocal { index: 0 }, 123);

        let function = Function::new_script(chunk);

        let mut vm = VM::new();

        let mut frame = Frame::new(Arc::new(function));
        frame.stack_offset = 1;

        // This is the variable from the previous frame
        vm.stack.push(Value::Nil);

        // The starting value of our local
        vm.stack.push(Value::Double(42.0));
        // The new value
        vm.stack.push(Value::Double(12.0));

        vm.interpret_frame(frame).unwrap();

        assert_eq!(3, vm.stack.len());
        assert_eq!(Value::Nil, vm.stack[0]);
        assert_eq!(Value::Double(12.0), vm.stack[1]);
        assert_eq!(Value::Double(12.0), vm.stack[2]);
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

        let mut vm = VM::new_from_settings(VMSettings::test_default());
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

        let mut vm = VM::new_from_settings(VMSettings::test_default());
        vm.interpret(function).unwrap();
        assert!(vm.captured_prints.is_empty());
        assert!(vm.is_stack_empty())
    }

    #[test]
    fn calls() {
        let inner_chunk = {
            let mut chunk = Chunk::new();
            chunk.write(Instruction::GetLocal { index: 0 }, 100);
            chunk.write(Instruction::Print, 101);
            chunk
        };

        let mut chunk = Chunk::new();
        chunk.write_constant(
            Value::Function(Arc::new(Function {
                arity: 1,
                chunk: inner_chunk,
                name: Some("f".to_string()),
            })),
            124,
        );
        chunk.write_constant(Value::Double(42.2), 123);
        chunk.write(Instruction::GetLocal { index: 0 }, 123);
        chunk.write(Instruction::GetLocal { index: 1 }, 123);
        chunk.write(Instruction::Call { arg_count: 1 }, 124);

        let mut vm = VM::new_from_settings(VMSettings::test_default());
        vm.interpret(Function {
            arity: 0,
            chunk: chunk,
            name: None,
        })
        .unwrap();
        assert_eq!("42.2", vm.captured_prints[0]);
        println!("{:?}", vm.stack);
        assert!(vm.is_stack_empty())
    }

    #[test]
    fn calls_wrong_arguments() {
        let inner_chunk = {
            let mut chunk = Chunk::new();
            chunk.write(Instruction::GetLocal { index: 0 }, 100);
            chunk.write(Instruction::Print, 101);
            chunk
        };

        let mut chunk = Chunk::new();
        chunk.write_constant(
            Value::Function(Arc::new(Function {
                arity: 1,
                chunk: inner_chunk,
                name: Some("f".to_string()),
            })),
            124,
        );
        chunk.write(Instruction::GetLocal { index: 0 }, 123);
        chunk.write(Instruction::Call { arg_count: 0 }, 124);

        let mut vm = VM::new_from_settings(VMSettings::test_default());
        let error = vm
            .interpret(Function {
                arity: 0,
                chunk: chunk,
                name: None,
            })
            .unwrap_err();
        assert_eq!(InterpretErrors::IncorrectArgumentCount(1, 0), error);
    }
}
