use crate::utils::BackEnd;

pub struct CompilerBackEnd {}

impl BackEnd for CompilerBackEnd {
    fn execute_single_line(&mut self, _line: &str) -> Result<(), String> {
        Ok(())
    }

    fn execute_script(&mut self, _script: &str) -> Result<(), String> {
        Ok(())
    }
}

impl CompilerBackEnd {
    pub fn init() -> CompilerBackEnd {
        CompilerBackEnd {}
    }
}
