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

    fn pop_as_falsey(&mut self) -> Result<bool, InterpretError> {
        if let Some(v) = self.pop() {
            match v {
                OpValue::Double(_) => Err(InterpretError::RuntimeError),
                OpValue::Boolean(v) => Ok(v == false),
                OpValue::Nil => Ok(true),
            }
        } else {
            Err(InterpretError::RuntimeError)
        }
    }

    fn pop_as_boolean(&mut self) -> Result<bool, InterpretError> {
        if let Some(v) = self.pop() {
            if let Some(v) = v.as_boolean() {
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
                OpCode::Not => {
                    let v = self.pop_as_falsey()?;
                    self.push(OpValue::Boolean(v));
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
                OpCode::Equal => {
                    let v2 = self.pop();
                    let v1 = self.pop();
                    self.push(OpValue::Boolean(v1 == v2));
                }
                OpCode::Greater => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Boolean(v1 > v2));
                }
                OpCode::Less => {
                    let v2 = self.pop_as_double()?;
                    let v1 = self.pop_as_double()?;
                    self.push(OpValue::Boolean(v1 < v2));
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
        assert!(vm.interpret(&chunk).is_ok());
    }

    fn assert_runtime_error(script: &str) {
        let chunk = compile(script).unwrap();
        let mut vm = VirtualMachine::new();
        assert!(vm.interpret(&chunk).is_err());
    }

    fn execute_script(script: &str) -> Option<OpValue> {
        let chunk = compile(script).unwrap();
        let mut vm = VirtualMachine::new();
        assert!(vm.interpret(&chunk).is_ok());
        vm.stack.first().copied()
    }

    #[test]
    fn negate() {
        assert_approx_eq!(-1.2, execute_script("-1.2;").unwrap().as_double().unwrap());
    }

    #[test]
    fn negate_boolean() {
        assert_runtime_error("-true;");
    }

    #[test]
    fn math_operations() {
        for (op, expected) in &[("+", 3.0), ("-", -1.0), ("*", 2.0), ("/", 0.5)] {
            assert_approx_eq!(expected, execute_script(&format!("1 {op} 2;")).unwrap().as_double().unwrap());
        }
    }

    #[test]
    fn add_boolean() {
        assert_runtime_error("1 + true;");
    }

    #[test]
    fn not_number() {
        assert_runtime_error("!42;");
    }

    #[test]
    fn not_boolean() {
        assert_eq!(Some(false), execute_script("!true;").unwrap().as_boolean());
    }

    #[test]
    fn equality() {
        assert_eq!(Some(true), execute_script("4 > 3;").unwrap().as_boolean());
        assert_eq!(Some(true), execute_script("4 >= 3;").unwrap().as_boolean());
        assert_eq!(Some(false), execute_script("4 < 3;").unwrap().as_boolean());
        assert_eq!(Some(false), execute_script("4 <= 3;").unwrap().as_boolean());
        assert_eq!(Some(false), execute_script("4 == 3;").unwrap().as_boolean());
    }

    #[test]
    fn value_smoke() {
        assert_eq!(Some(true), execute_script("!(5 - 4 > 3 * 2 == !nil);").unwrap().as_boolean());
    }
}