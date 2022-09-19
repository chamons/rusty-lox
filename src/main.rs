#![allow(
    clippy::ptr_arg,
    clippy::collapsible_else_if,
    clippy::len_zero,
    clippy::nonminimal_bool,
    clippy::module_inception
)]

mod bytecode;
mod interpreter;
mod parser;
mod utils;

use bytecode::BytecodeFrontEnd;
use interpreter::InterpreterFrontEnd;
use utils::die;

#[macro_use]
extern crate lazy_static;

use std::{
    env, fs,
    io::{self, Write},
};

trait FrontEnd {
    fn execute_single_line(&mut self, line: &str) -> Result<(), String>;
    fn execute_script(&mut self, script: &str) -> Result<(), String>;
}

fn run_file(target: &mut dyn FrontEnd, path: &str) {
    match fs::read_to_string(path) {
        Ok(script) => {
            if let Err(e) = target.execute_script(&script) {
                println!("Error: {e}");
            }
        }
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt(target: &mut dyn FrontEnd) {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush().expect("Stdout flush");
        stdin.read_line(&mut line).expect("Error reading console");
        if let Err(e) = target.execute_single_line(&line) {
            println!("Error: {e}");
        }
        line.clear();
    }
}

fn main() {
    let mut args = env::args();
    let mut interpreter: Box<dyn FrontEnd> = if env::var("USE_INTERPRETER").is_ok() {
        Box::new(InterpreterFrontEnd::init(Box::new(|p| println!("{}", p))))
    } else {
        Box::new(BytecodeFrontEnd::new())
    };

    // First argument is our program name
    match args.len() - 1 {
        0 => run_prompt(interpreter.as_mut()),
        1 => run_file(interpreter.as_mut(), &args.nth(1).unwrap()),
        _ => {
            die("Usage: rusty-lox [script]");
        }
    }
}
