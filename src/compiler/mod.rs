use scanner::Scanner;

mod scanner;
mod source;
mod token;

pub fn compile(source: &String) -> eyre::Result<()> {
    let scanner = Scanner::new(source);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compile_hello_world() {
        compile(&"1 + 2".to_string()).unwrap();
    }
}
