use thiserror::Error;

use super::*;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Compile Error")]
    CompileError,
}

pub fn compile(_source: &str) -> Result<Chunk, CompilerError> {
    let chunk = Chunk::new();

    Ok(chunk)
}
