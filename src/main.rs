mod interpreter;
mod utils;

use std::{env, fs};
use utils::die;

fn run_file(path: &str) {
    let file = fs::read_to_string(path);
    match file {
        Ok(file) => {
            let errors = interpreter::run(&file);
            for error in errors {
                println!("{}", error);
                die("");
            }
        }
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        println!("> ");
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(_) => {
                let errors = interpreter::run(&line);
                for error in errors {
                    println!("{}", error);
                }
            }
            Err(err) => die(&format!("Error reading console due to {}", err)),
        }
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
