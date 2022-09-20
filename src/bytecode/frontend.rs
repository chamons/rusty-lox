use crate::FrontEnd;

use super::VirtualMachine;

pub struct BytecodeFrontEnd<'a> {
    vm: VirtualMachine<'a>,
}

impl<'a> BytecodeFrontEnd<'a> {
    pub fn new() -> Self {
        BytecodeFrontEnd { vm: VirtualMachine::new() }
    }
}

impl<'a> FrontEnd for BytecodeFrontEnd<'a> {
    fn execute_single_line(&mut self, _line: &str) -> Result<(), String> {
        todo!()
    }

    fn execute_script(&mut self, _script: &str) -> Result<(), String> {
        todo!()
    }
}
