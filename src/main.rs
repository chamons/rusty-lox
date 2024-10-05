use vm::VM;

mod bytecode;
mod vm;

mod tracing;

fn main() {
    tracing::configure_default_tracing();

    let vm = VM::default();
}
