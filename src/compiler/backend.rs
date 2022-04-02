use anyhow::{anyhow, Result};

use crate::{
    parser::{Parser, Scanner},
    utils::BackEnd,
};

use super::compiler::Compiler;

pub struct CompilerBackEnd {
    compiler: Compiler,
}

impl BackEnd for CompilerBackEnd {
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

        let wasm = self.compiler.compile(&statements);
        super::runtime::execute(&wasm)?;

        Ok(())
    }
}

impl CompilerBackEnd {
    pub fn init() -> CompilerBackEnd {
        CompilerBackEnd { compiler: Compiler::init() }
    }
}
