mod interpreter;
mod parser;
mod utils;

use utils::die;

#[macro_use]
extern crate lazy_static;

use std::{
    cell::RefCell,
    env, fs,
    io::{self, Write},
    rc::Rc,
};

use interpreter::{Interpreter, Resolver};
use parser::{Parser, Scanner};

fn run_file(path: &str) {
    let file = fs::read_to_string(path);
    match file {
        Ok(file) => interpreter::run_script(&file),
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut line = String::new();
    let interpreter = Rc::new(RefCell::new(Interpreter::init(Box::new(|p| println!("{}", p)))));
    let mut resolver = Resolver::init(&interpreter);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        match stdin.read_line(&mut line) {
            Ok(_) => {
                let mut scanner = Scanner::init(&line);
                let (tokens, errors) = scanner.scan_tokens();
                if errors.len() > 0 {
                    for e in errors {
                        println!("{}", e);
                    }
                    return;
                }
                let mut parser = Parser::init(tokens);
                match parser.parse() {
                    Ok(statements) => {
                        match resolver.resolve_statements(&statements) {
                            Err(e) => println!("Error: {}", e),
                            _ => {}
                        }

                        match interpreter.borrow_mut().execute(&statements) {
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                            _ => {}
                        }
                    }
                    Err(err) => {
                        // If we fail parsing as a statement, try an expression and print the value if so
                        parser.reset_position();
                        match parser.parse_single_expression() {
                            Ok(expression) => {
                                match interpreter.borrow_mut().execute_expression(&expression) {
                                    Ok(result) => println!("{}", result),
                                    Err(err) => println!("Error: {}", err),
                                };
                            }
                            Err(_) => println!("Error: {}", err),
                        };
                    }
                }
            }
            Err(err) => die(&format!("Error reading console due to {}", err)),
        }
        line.clear();
    }
}

fn main() {
    let mut args = env::args();

    // First argument is our program name
    match args.len() - 1 {
        0 => run_prompt(),
        1 => run_file(&args.nth(1).unwrap()),
        _ => {
            die("Usage: rusty-lox [script]");
        }
    }
}
