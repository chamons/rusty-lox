mod scanner;

pub fn compile(source: &String) -> eyre::Result<()> {
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
