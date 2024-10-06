use crate::{
    bytecode::{Chunk, Instruction, Value},
    compiler::parser::Parser,
};

use super::tokens::token::TokenType;

struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Self { chunk: Chunk::new() }
    }

    pub fn compile(&mut self, source: &str) -> eyre::Result<Chunk> {
        self.chunk = Chunk::new();

        let mut parser = Parser::new(source)?;

        parser.advance()?;

        self.emit_return(parser.current.line);
        self.consume(&mut parser, TokenType::Eof, "Expect end of expression.")?;

        Ok(std::mem::take(&mut self.chunk))
    }

    fn emit_return(&mut self, line: u32) {
        self.chunk.write(Instruction::Return, line);
    }

    fn emit_constant(&mut self, value: Value, line: u32) {
        self.chunk.write_constant(value, line);
    }

    fn number(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        // parser.previous
        Ok(())
    }

    fn expression(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        Ok(())
    }

    fn consume(&mut self, parser: &mut Parser, token: TokenType, message: &str) -> eyre::Result<()> {
        if parser.current.token_type == token {
            parser.advance()?;
            return Ok(());
        }

        Err(eyre::eyre!(message.to_string()))
    }
}
