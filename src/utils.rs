use std::{fmt::Display, process::exit};

pub fn die(error: &str) -> ! {
    if error.len() > 0 {
        eprintln!("{}", error);
    }
    exit(-1);
}

#[derive(Debug, Clone)]
pub struct ScannerError {
    line: u32,
    location: String,
    message: String,
}

impl ScannerError {
    pub fn init(line: u32, location: &str, message: &str) -> Self {
        ScannerError {
            line,
            location: location.to_string(),
            message: message.to_string(),
        }
    }
}

impl Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error{}: {}", self.line, self.location, self.message)
    }
}
