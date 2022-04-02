use std::{cell::RefCell, rc::Rc};

use crate::parser::{Parser, Scanner};

use super::{Interpreter, InterpreterLiteral, Resolver};

pub struct FrontEnd {
    interpreter: Rc<RefCell<Interpreter>>,
    resolver: Resolver,
}

impl FrontEnd {
    pub fn init(print: Box<dyn FnMut(&InterpreterLiteral)>) -> FrontEnd {
        let interpreter = Rc::new(RefCell::new(Interpreter::init(print)));
        FrontEnd {
            resolver: Resolver::init(&interpreter),
            interpreter,
        }
    }

    pub fn execute_single_line(&mut self, line: &str) -> Result<(), String> {
        let mut scanner = Scanner::init(&line);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(format!("{:?}", errors));
        }
        let mut parser = Parser::init(tokens);
        match parser.parse() {
            Ok(statements) => {
                self.resolver.resolve_statements(&statements)?;
                self.interpreter.borrow_mut().execute(&statements)?;
            }
            Err(_) => {
                // If we fail parsing as a statement, try an expression and print the value if so
                parser.reset_position();
                let expression = parser.parse_single_expression()?;
                let result = self.interpreter.borrow_mut().execute_expression(&expression)?;
                println!("{}", result);
            }
        };
        Ok(())
    }

    pub fn execute_script(&mut self, script: &str) -> Result<(), String> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(format!("{:?}", errors));
        }
        let mut parser = Parser::init(tokens);
        let statements = parser.parse()?;
        self.resolver.resolve_statements(&statements)?;
        self.interpreter.borrow_mut().execute(&statements)?;
        Ok(())
    }
}
