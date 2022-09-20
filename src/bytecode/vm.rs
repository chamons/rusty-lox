use thiserror::Error;

use super::*;

#[derive(Error, Debug)]
enum InterpretError {
    #[error("Compile Error")]
    CompileError,
    #[error("Runtime Error")]
    RuntimeError,
}

pub struct VirtualMachine {
    stack: Vec<OpValue>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine { stack: vec![] }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        self.run(chunk)
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self, value: OpValue) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<OpValue> {
        self.stack.pop()
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        let mut ip: usize = 0;
        loop {
            let instruction = &chunk.code[ip];
            if cfg!(debug_assertions) {
                print!("        ");
                for s in &self.stack {
                    print!("[ {:?} ]", s);
                }
                println!();
                println!("{}", instruction.disassemble(&chunk));
            }
            match instruction {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant(index) => {
                    self.push(chunk.values[*index]);
                }
                OpCode::Negate => match self.pop() {
                    Some(OpValue::Double(v)) => {
                        self.push(OpValue::Double(v * -1.0));
                    }
                    _ => {
                        return Err(InterpretError::RuntimeError);
                    }
                },
            }
            ip += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_smoke_test() {
        let mut chunk = Chunk::new();
        let index = chunk.write_value(OpValue::Double(1.2));
        chunk.write(OpCode::Constant(index), 1);
        chunk.write(OpCode::Return, 1);
        let mut vm = VirtualMachine::new();
        assert!(vm.interpret(&mut chunk).is_ok());
    }

    #[test]
    fn negate() {
        let mut chunk = Chunk::new();
        let index = chunk.write_value(OpValue::Double(1.2));
        chunk.write(OpCode::Constant(index), 1);
        chunk.write(OpCode::Negate, 2);
        chunk.write(OpCode::Return, 3);
        let mut vm = VirtualMachine::new();
        assert!(vm.interpret(&mut chunk).is_ok());
        assert_eq!(Some(&OpValue::Double(-1.2)), vm.stack.first().as_deref());
    }
}
