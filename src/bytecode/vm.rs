use thiserror::Error;

use super::*;

#[derive(Error, Debug)]
pub enum InterpretError {
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

    fn pop_as_double(&mut self) -> Result<f64, InterpretError> {
        if let Some(v) = self.pop() {
            if let Some(v) = v.as_double() {
                return Ok(v);
            }
        }
        Err(InterpretError::RuntimeError)
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
                OpCode::Negate => {
                    let v = self.pop_as_double()?;
                    self.push(OpValue::Double(v * -1.0));
                }
                OpCode::Add => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Double(v1 + v2));
                }
                OpCode::Subtract => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Double(v1 - v2));
                }
                OpCode::Multiply => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Double(v1 * v2));
                }
                OpCode::Divide => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Double(v1 / v2));
                }
            }
            ip += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

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
        assert_approx_eq!(-1.2, vm.stack.first().unwrap().as_double().unwrap());
    }

    #[test]
    fn math_operations() {
        for (op, expected) in &[(OpCode::Add, 3.0), (OpCode::Subtract, -1.0), (OpCode::Multiply, 2.0), (OpCode::Divide, 0.5)] {
            let mut chunk = Chunk::new();
            let index1 = chunk.write_value(OpValue::Double(1.0));
            let index2 = chunk.write_value(OpValue::Double(2.0));
            chunk.write(OpCode::Constant(index1), 1);
            chunk.write(OpCode::Constant(index2), 2);
            chunk.write(*op, 3);
            chunk.write(OpCode::Return, 3);
            let mut vm = VirtualMachine::new();
            assert!(vm.interpret(&mut chunk).is_ok());
            assert_approx_eq!(expected, vm.stack.first().unwrap().as_double().unwrap());
        }
    }
}
