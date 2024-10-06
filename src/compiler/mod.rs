use parser::scanner::Scanner;

mod parser;

pub fn compile(source: &str) -> eyre::Result<()> {
    let _scanner = Scanner::new(source);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compile_hello_world() {
        compile("1 + 2").unwrap();
    }
}
