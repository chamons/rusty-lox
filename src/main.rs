mod interpreter;
mod parser;
mod utils;

use interpreter::FrontEnd;
use utils::die;

#[macro_use]
extern crate lazy_static;

use std::{
    env, fs,
    io::{self, Write},
};

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(script) => {
            let mut interpreter = FrontEnd::init(Box::new(|p| println!("{}", p)));
            if let Err(e) = interpreter.execute_script(&script) {
                println!("Error: {e}");
            }
        }
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut interpreter = FrontEnd::init(Box::new(|p| println!("{}", p)));

    loop {
        print!("> ");
        io::stdout().flush().expect("Stdout flush");
        stdin.read_line(&mut line).expect("Error reading console");
        if let Err(e) = interpreter.execute_single_line(&line) {
            println!("Error: {e}");
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
