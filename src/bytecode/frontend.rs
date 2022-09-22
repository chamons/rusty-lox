use crate::FrontEnd;

use super::{compile, VirtualMachine};

pub struct BytecodeFrontEnd {}

impl BytecodeFrontEnd {
    pub fn new() -> Self {
        BytecodeFrontEnd {}
    }
}

impl FrontEnd for BytecodeFrontEnd {
    fn execute_single_line(&mut self, line: &str) -> anyhow::Result<()> {
        self.execute_script(line)
    }

    fn execute_script(&mut self, script: &str) -> anyhow::Result<()> {
        let (chunk, strings) = compile(script)?;
        let mut vm = VirtualMachine::new().with_strings(strings);
        vm.interpret(&chunk)?;
        Ok(())
    }
}
