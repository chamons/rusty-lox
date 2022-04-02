use std::str;
use std::time::SystemTime;

use anyhow::Result;

use wasmtime::*;

struct RuntimeState {
    logs: Vec<String>,
}

impl RuntimeState {
    pub fn init() -> RuntimeState {
        RuntimeState { logs: vec![] }
    }
}

pub fn execute(binary: &[u8], print: &mut Box<dyn FnMut(&str)>) -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, binary)?;
    let state = RuntimeState::init();
    let mut store = Store::new(&engine, state);

    let clock = Func::wrap(&mut store, || -> f64 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
    });

    let log_str = Func::wrap(&mut store, |mut caller: Caller<'_, RuntimeState>, ptr: i32, len: i32| {
        let string = {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return Err(Trap::new("failed to find host memory")),
            };
            let data = mem.data(&caller).get(ptr as u32 as usize..).and_then(|arr| arr.get(..len as u32 as usize));
            match data {
                Some(data) => match str::from_utf8(data) {
                    Ok(s) => s.to_string(),
                    Err(_) => return Err(Trap::new("invalid utf-8")),
                },
                None => return Err(Trap::new("pointer/length out of bounds")),
            }
        };
        caller.data_mut().logs.push(string);
        Ok(())
    });

    let log_num = Func::wrap(&mut store, |mut caller: Caller<'_, RuntimeState>, num: f64| {
        caller.data_mut().logs.push(format!("{}", num));
        Ok(())
    });

    let imports = [clock.into(), log_str.into(), log_num.into()];

    // This executes the start() function
    Instance::new(&mut store, &module, &imports)?;

    for l in &store.data().logs {
        print(l);
    }

    Ok(())
}
