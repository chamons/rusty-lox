use std::{cell::RefCell, rc::Rc, time::SystemTime};

use super::environment::Environment;
use crate::{
    interpreter::{Interpreter, InterpreterLiteral},
    parser::{ChildStatement, Token},
};

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<InterpreterLiteral>) -> Result<InterpreterLiteral, &'static str>;
    fn name(&self) -> &str;
    fn arity(&self) -> u32;
    fn duplicate(&self) -> Box<dyn Callable>;
}

pub struct ClockPrimitive {}

impl ClockPrimitive {
    pub fn init() -> Self {
        ClockPrimitive {}
    }
}

impl Callable for ClockPrimitive {
    fn call(&self, _: &mut Interpreter, _: &Vec<InterpreterLiteral>) -> Result<InterpreterLiteral, &'static str> {
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

    fn duplicate(&self) -> Box<dyn Callable> {
        Box::new(ClockPrimitive::init())
    }
}

pub struct UserFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<ChildStatement>,
    closure: Rc<RefCell<Environment>>,
}

impl UserFunction {
    pub fn init(name: &Token, params: &Vec<Token>, body: &Vec<ChildStatement>, closure: &Rc<RefCell<Environment>>) -> Self {
        UserFunction {
            name: name.clone(),
            params: params.clone(),
            body: body.clone(),
            closure: Rc::clone(closure),
        }
    }
}

impl Callable for UserFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<InterpreterLiteral>) -> Result<InterpreterLiteral, &'static str> {
        let environment = Rc::new(RefCell::new(Environment::init_with_parent(&self.closure)));
        for (i, arg) in self.params.iter().enumerate() {
            environment.borrow_mut().define(&arg.lexme, arguments[i].clone());
        }
        interpreter.execute_block(&self.body, environment)?;
        Ok(InterpreterLiteral::Nil)
    }

    fn name(&self) -> &str {
        &self.name.lexme
    }

    fn arity(&self) -> u32 {
        self.params.len() as u32
    }

    fn duplicate(&self) -> Box<dyn Callable> {
        Box::new(UserFunction::init(&self.name, &self.params, &self.body, &self.closure))
    }
}
