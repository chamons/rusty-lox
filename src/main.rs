mod expressions;
pub mod interpreter;
mod parser;
pub mod tokens;
mod utils;

#[macro_use]
extern crate lazy_static;

use std::{env, fs};
use utils::die;

fn run_file(path: &str) {
    let file = fs::read_to_string(path);
    match file {
        Ok(file) => interpreter::run(&file),
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        println!("> ");
        match stdin.read_line(&mut line) {
            Ok(_) => interpreter::run(&line),
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
