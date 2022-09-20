use thiserror::Error;

use super::Chunk;

#[derive(Error, Debug)]
enum InterpretError {
    #[error("Compile Error")]
    CompileError,
    #[error("Runtime Error")]
    RuntimeError,
}

pub struct VirtualMachine<'a> {
    chunk: Option<&'a Chunk>,
    ip: usize,
}

impl<'a> VirtualMachine<'a> {
    pub fn new() -> Self {
        VirtualMachine { chunk: None, ip: 0 }
    }

    pub fn interpret(&mut self, chunk: &'a mut Chunk) -> Result<(), InterpretError> {
        self.chunk = Some(chunk);
        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        if let Some(chunk) = &self.chunk {
            loop {
                let instruction = &chunk.code[self.ip];
                if cfg!(debug_assertions) {
                    println!("{}", instruction.disassemble(&chunk));
                }
                match instruction {
                    super::OpCode::Return => return Ok(()),
                    super::OpCode::Constant(index) => {
                        println!("{:?}", chunk.values[*index]);
                    }
                }
                self.ip += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::*;

    #[test]
    fn vm_smoke_test() {
        let mut chunk = Chunk::new();
        let index = chunk.write_value(OpValue::Double(1.2));
        chunk.write(OpCode::Constant(index), 1);
        chunk.write(OpCode::Return, 1);
        let mut vm = VirtualMachine::new();
        assert!(vm.interpret(&mut chunk).is_ok());
    }
}
