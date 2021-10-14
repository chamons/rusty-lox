use std::time::SystemTime;

use crate::interpreter::InterpreterLiteral;

pub trait Callable {
    fn call(&self, arguments: &Vec<InterpreterLiteral>) -> Result<InterpreterLiteral, &'static str>;
    fn name(&self) -> &str;
    fn arity(&self) -> u32;
}

pub struct ClockPrimitive {}

impl ClockPrimitive {
    pub fn init() -> Self {
        ClockPrimitive {}
    }
}

impl Callable for ClockPrimitive {
    fn call(&self, _: &Vec<InterpreterLiteral>) -> Result<InterpreterLiteral, &'static str> {
        Ok(InterpreterLiteral::Number(
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64(),
        ))
    }

    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> u32 {
        0
    }
}
