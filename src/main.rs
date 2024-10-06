#![allow(dead_code, unreachable_patterns)]

use compiler::compile;
use eyre::eyre;
use std::{env::args, fs, io::Write};

use vm::VM;

mod bytecode;
mod compiler;
mod tracing;
mod utils;
mod vm;

fn repl() -> eyre::Result<()> {
    let mut vm = VM::default();

    println!("Type exit to quit");
    println!();
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        utils::trim_newline(&mut line);

        if line == "exit" {
            return Ok(());
        }

        let chunk = compile(&line)?;

        if let Err(err) = vm.interpret(&chunk) {
            eprintln!("{err:?}")
        }
    }
}

fn run_file(path: String) -> eyre::Result<()> {
    let mut vm = VM::default();

    let source = fs::read_to_string(path)?;
    let chunk = compile(&source)?;

    Ok(vm.interpret(&chunk)?)
}

fn main() -> eyre::Result<()> {
    tracing::configure_tracing(::tracing::level_filters::LevelFilter::TRACE);

    match args().len() {
        1 => repl(),
        2 => run_file(args().nth(1).unwrap().to_string()),
        _ => Err(eyre!("Usage: rusty-lox [path]")),
    }
}
