use crate::expressions::print_tree;
use crate::parser::*;
use crate::tokens::*;

pub fn run(script: &str) {
    let mut scanner = Scanner::init(script);
    let (tokens, errors) = scanner.scan_tokens();
    if errors.len() > 0 {
        return;
    }

    let mut parser = Parser::init(tokens);
    match parser.parse() {
        Ok(expression) => {
            println!("Tree: {}", print_tree(expression));
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
