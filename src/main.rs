use eyre::eyre;
use std::{env::args, fs, io::Write};

use vm::VM;

mod bytecode;
mod vm;

mod tracing;
mod utils;

fn repl() -> eyre::Result<()> {
    let mut vm = VM::default();

    println!("Type exit to quit");
    println!("");
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        utils::trim_newline(&mut line);

        if line == "exit" {
            return Ok(());
        }

        if let Err(err) = vm.interpret(&line) {
            eprintln!("{err:?}")
        }
    }
}

fn run_file(path: String) -> eyre::Result<()> {
    let mut vm = VM::default();

    let source = fs::read_to_string(path)?;
    Ok(vm.interpret(&source)?)
}

fn main() -> eyre::Result<()> {
    tracing::configure_default_tracing();

    match args().len() {
        1 => repl(),
        2 => run_file(args().skip(1).next().unwrap().to_string()),
        _ => Err(eyre!("Usage: rusty-lox [path]")),
    }
}
