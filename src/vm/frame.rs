use crate::bytecode::{Instruction, Value};

use super::{Function, InterpretErrors};

#[derive(Debug, Default)]
pub struct Frame {
    pub function: Function,
    pub ip: usize,
    pub stack_offset: usize,
}

impl Frame {
    pub fn new(function: Function) -> Self {
        {
            Self {
                function,
                ip: 0,
                stack_offset: 0,
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

    pub fn fetch_constant_name(&self, index: usize) -> Result<String, InterpretErrors> {
        match self.function.chunk.constant(index) {
            Value::String(name) => Ok(name.clone()),
            _ => Err(InterpretErrors::InvalidRuntimeType),
        }
    }
}
