use compiler::Compiler;

use crate::bytecode::Chunk;

pub mod compiler;
pub mod parser;
pub mod tokens;

pub fn compile(source: &str) -> eyre::Result<Chunk> {
    let mut compiler = Compiler::new();
    Ok(compiler.compile(source)?)
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compile_hello_world() {
        compile("1 + 2").unwrap();
    }
}
