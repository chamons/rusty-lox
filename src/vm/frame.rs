use smallvec::SmallVec;

use crate::bytecode::{Instruction, Value};

use super::{Function, InterpretErrors};

#[derive(Debug, Default)]
pub struct Frame {
    pub function: Function,
    pub ip: usize,
    pub stack: SmallVec<[Value; 4]>,
}

impl Frame {
    pub fn new(function: Function) -> Self {
        {
            Self {
                function,
                ip: 0,
                stack: SmallVec::new(),
            }
        }
    }

    pub fn next_instruction(&mut self) -> Option<Instruction> {
        let instruction = self.function.chunk.code.get(self.ip).cloned();
        self.ip += 1;
        instruction
    }

    pub fn constant(&self, index: usize) -> Value {
        self.function.chunk.constant(index as usize).clone()
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

    pub fn fetch_constant_name(&self, index: usize) -> Result<String, InterpretErrors> {
        match self.function.chunk.constant(index) {
            Value::String(name) => Ok(name.clone()),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bytecode::Value,
        vm::{Frame, Function},
    };

    #[test]
    fn falsey() {
        let mut frame = Frame::new(Function::new());
        frame.stack.push(Value::Double(1.2));
        frame.stack.push(Value::Double(0.0));
        frame.stack.push(Value::Nil);
        frame.stack.push(Value::Bool(true));
        frame.stack.push(Value::Bool(false));
        assert!(frame.pop_falsey().unwrap());
        assert!(!frame.pop_falsey().unwrap());
        assert!(frame.pop_falsey().unwrap());
        assert!(!frame.pop_falsey().unwrap());
        assert!(!frame.pop_falsey().unwrap());
    }
}
