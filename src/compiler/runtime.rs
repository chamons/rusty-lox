use std::time::SystemTime;

use anyhow::Result;

use wasmtime::*;

struct RuntimeState {}

pub fn execute(binary: &[u8]) -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, binary)?;

    let mut store = Store::new(&engine, RuntimeState {});

    let clock_func = Func::wrap(&mut store, |mut _caller: Caller<'_, RuntimeState>| -> f64 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
    });

    let imports = [clock_func.into()];

    // This executes the start() function
    Instance::new(&mut store, &module, &imports)?;

    Ok(())
}
