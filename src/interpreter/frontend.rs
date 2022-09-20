use anyhow::anyhow;
use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::{Parser, Scanner},
    FrontEnd,
};

use super::{Interpreter, InterpreterLiteral, Resolver};

pub struct InterpreterFrontEnd {
    interpreter: Rc<RefCell<Interpreter>>,
    resolver: Resolver,
}

impl InterpreterFrontEnd {
    pub fn init(print: Box<dyn FnMut(&InterpreterLiteral)>) -> InterpreterFrontEnd {
        let interpreter = Rc::new(RefCell::new(Interpreter::init(print)));
        InterpreterFrontEnd {
            resolver: Resolver::init(&interpreter),
            interpreter,
        }
    }
}

impl FrontEnd for InterpreterFrontEnd {
    fn execute_single_line(&mut self, line: &str) -> anyhow::Result<()> {
        let mut scanner = Scanner::init(line);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(anyhow!("{:?}", errors));
        }
        let mut parser = Parser::init(tokens);
        match parser.parse() {
            Ok(statements) => {
                self.resolver.resolve_statements(&statements).map_err(anyhow::Error::msg)?;
                self.interpreter.borrow_mut().execute(&statements).map_err(anyhow::Error::msg)?;
            }
            Err(_) => {
                // If we fail parsing as a statement, try an expression and print the value if so
                parser.reset_position();
                let expression = parser.parse_single_expression().map_err(anyhow::Error::msg)?;
                let result = self.interpreter.borrow_mut().execute_expression(&expression).map_err(anyhow::Error::msg)?;
                println!("{}", result);
            }
        };
        Ok(())
    }

    fn execute_script(&mut self, script: &str) -> anyhow::Result<()> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(anyhow!("{:?}", errors));
        }
        let mut parser = Parser::init(tokens);
        let statements = parser.parse().map_err(anyhow::Error::msg)?;
        self.resolver.resolve_statements(&statements).map_err(anyhow::Error::msg)?;
        self.interpreter.borrow_mut().execute(&statements).map_err(anyhow::Error::msg)?;
        Ok(())
    }
}
