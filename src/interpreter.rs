use crate::{tokens::Token, utils::ScannerError};

pub fn run(script: &str) -> (Vec<Token>, Vec<ScannerError>) {
    crate::tokens::run(script)
}
