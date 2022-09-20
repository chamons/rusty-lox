use crate::FrontEnd;

use super::{compile, VirtualMachine};

pub struct BytecodeFrontEnd {
    vm: VirtualMachine,
}

impl BytecodeFrontEnd {
    pub fn new() -> Self {
        BytecodeFrontEnd { vm: VirtualMachine::new() }
    }
}

impl FrontEnd for BytecodeFrontEnd {
    fn execute_single_line(&mut self, line: &str) -> anyhow::Result<()> {
        self.execute_script(line)
    }

    fn execute_script(&mut self, script: &str) -> anyhow::Result<()> {
        let chunk = compile(script)?;
        self.vm.interpret(&chunk)?;
        Ok(())
    }
}
