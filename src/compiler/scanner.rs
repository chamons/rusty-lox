#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a String,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Self { source }
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;

    #[test]
    fn compile_hello_world() {
        let source = "1 + 2".to_string();
        let scanner = Scanner::new(&source);
    }
}
