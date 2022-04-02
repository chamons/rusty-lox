use anyhow::Result;

use wasmtime::*;

fn execute(binary: &[u8]) -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, binary)?;

    Ok(())
}
