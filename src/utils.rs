use std::process::exit;

pub trait BackEnd {
    fn execute_single_line(&mut self, line: &str) -> Result<(), String>;
    fn execute_script(&mut self, script: &str) -> Result<(), String>;
}

pub fn die(error: &str) -> ! {
    if error.len() > 0 {
        eprintln!("{}", error);
    }
    exit(-1);
}
