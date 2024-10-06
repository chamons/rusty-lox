use std::{error::Error, fmt::Display};

use super::tokens::{
    scanner::Scanner,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub struct ParserError {
    token: Option<Token>,
    err: eyre::Error,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(token) = self.token.as_ref() {
            f.write_fmt(format_args!("[line {}] Error", token.line))?;

            if matches!(token.token_type, TokenType::Eof) {
                f.write_str(" at end")?;
            }
        }

        f.write_fmt(format_args!("{}", self.err))?;

        // TODO - Improve Error Messages

        Ok(())
    }
}

pub struct Parser<'a> {
    pub previous: Token,
    pub current: Token,
    scanner: Scanner<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Parser<'a>, ParserError> {
        let mut scanner = Scanner::new(source);

        let first = scanner.scan().map_err(|err| ParserError { err, token: None })?;

        Ok(Self {
            previous: first.clone(),
            current: first,
            scanner,
        })
    }

    pub fn advance(&mut self) -> Result<(), ParserError> {
        let next = self.scanner.scan().map_err(|err| ParserError {
            err,
            token: Some(self.previous.clone()),
        })?;

        self.previous = std::mem::replace(&mut self.current, next);

        Ok(())
    }
}
