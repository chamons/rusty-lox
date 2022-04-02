#![allow(
    clippy::ptr_arg,
    clippy::collapsible_else_if,
    clippy::len_zero,
    clippy::nonminimal_bool,
    clippy::module_inception
)]

use std::{
    fs,
    io::{self, Write},
};

use clap::Parser;

mod compiler;
mod interpreter;
mod parser;
mod utils;

use utils::die;

#[macro_use]
extern crate lazy_static;

use crate::utils::BackEnd;

fn run_file(back_end: &mut Box<dyn BackEnd>, path: &str) {
    match fs::read_to_string(path) {
        Ok(script) => {
            if let Err(e) = back_end.execute_script(&script) {
                println!("Error: {e}");
            }
        }
        Err(err) => die(&format!("Unable to read {} due to {}", path, err)),
    }
}

fn run_prompt(back_end: &mut Box<dyn BackEnd>) {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush().expect("Stdout flush");
        stdin.read_line(&mut line).expect("Error reading console");
        if let Err(e) = back_end.execute_single_line(&line) {
            println!("Error: {e}");
        }
        line.clear();
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Script to run
    #[clap(short, long)]
    script: Option<String>,

    /// Use interpreter instead of compiler
    #[clap(short, long)]
    use_interpreter: bool,
}

fn main() {
    let args = Args::parse();

    let mut back_end: Box<dyn BackEnd> = if args.use_interpreter {
        Box::new(interpreter::InterpreterBackEnd::init(Box::new(|p| println!("{}", p))))
    } else {
        Box::new(compiler::CompilerBackEnd::init(Box::new(|p| println!("{}", p))))
    };

    if let Some(script) = args.script {
        run_file(&mut back_end, &script);
    } else {
        run_prompt(&mut back_end);
    }
}
