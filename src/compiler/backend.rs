use anyhow::{anyhow, Result};

use crate::{
    parser::{Parser, Scanner},
    utils::BackEnd,
};

use super::compiler::Compiler;

pub struct CompilerBackEnd<'a> {
    compiler: Compiler<'a>,
    print: Box<dyn FnMut(&str)>,
}

impl<'a> BackEnd for CompilerBackEnd<'a> {
    fn execute_single_line(&mut self, _line: &str) -> Result<()> {
        todo!()
    }

    fn execute_script(&mut self, script: &str) -> Result<()> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(anyhow!(format!("{:?}", errors)));
        }
        let mut parser = Parser::init(tokens);
        let statements = parser.parse()?;

        let wasm = self.compiler.compile(&statements)?;
        super::runtime::execute(&wasm, &mut self.print)?;

        Ok(())
    }
}

impl<'a> CompilerBackEnd<'a> {
    pub fn init(print: Box<dyn FnMut(&str)>) -> CompilerBackEnd<'a> {
        CompilerBackEnd {
            compiler: Compiler::init(),
            print,
        }
    }
}
