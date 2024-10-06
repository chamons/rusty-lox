use compiler::Compiler;

use crate::bytecode::Chunk;

pub mod compiler;
pub mod parser;
pub mod tokens;

pub fn compile(source: &str) -> eyre::Result<Chunk> {
    let mut compiler = Compiler::new();
    Ok(compiler.compile(source)?)
}
