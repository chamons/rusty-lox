use crate::FrontEnd;

pub struct BytecodeFrontEnd {}

impl BytecodeFrontEnd {
    pub fn new() -> Self {
        BytecodeFrontEnd {}
    }
}

impl FrontEnd for BytecodeFrontEnd {
    fn execute_single_line(&mut self, _line: &str) -> Result<(), String> {
        todo!()
    }

    fn execute_script(&mut self, _script: &str) -> Result<(), String> {
        todo!()
    }
}
