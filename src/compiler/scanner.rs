use std::{iter::Peekable, ops::Index, str::Chars};

use itertools::{Itertools, MultiPeek};

use super::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a String,
    characters: Peekable<Chars<'a>>,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Self {
            source,
            characters: source.chars().peekable(),
            line: 1,
        }
    }

    pub fn scan(&mut self) -> eyre::Result<Token> {
        self.skip_whitespace();

        let c = match self.advance() {
            Some(c) => c,
            None => {
                return self.token(TokenType::EOF);
            }
        };

        match c {
            '(' => return self.token(TokenType::LeftParen),
            ')' => return self.token(TokenType::RightParen),
            '{' => return self.token(TokenType::LeftBrace),
            '}' => return self.token(TokenType::RightBrace),
            ';' => return self.token(TokenType::Semicolon),
            ',' => return self.token(TokenType::Comma),
            '.' => return self.token(TokenType::Dot),
            '-' => return self.token(TokenType::Minus),
            '+' => return self.token(TokenType::Plus),
            '/' => return self.token(TokenType::Slash),
            '*' => return self.token(TokenType::Star),
            '!' => {
                let r = if self.match_character('=') { TokenType::BangEqual } else { TokenType::Bang };
                return self.token(r);
            }
            '=' => {
                let r = if self.match_character('=') { TokenType::EqualEqual } else { TokenType::Equal };
                return self.token(r);
            }
            '<' => {
                let r = if self.match_character('=') { TokenType::LessEqual } else { TokenType::Less };
                return self.token(r);
            }
            '>' => {
                let r = if self.match_character('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                return self.token(r);
            }
            _ => {}
        }

        Err(eyre::eyre!("Unexpected character {c}"))
    }

    fn advance(&mut self) -> Option<char> {
        self.characters.next()
    }

    fn match_character(&mut self, expected: char) -> bool {
        match self.characters.peek() {
            Some(c) => {
                if *c == expected {
                    _ = self.characters.next();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.characters.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn token(&mut self, token_type: TokenType) -> eyre::Result<Token> {
        Ok(Token { token_type, line: self.line })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::compiler::token::TokenType;

    use super::Scanner;

    #[rstest]
    #[case("", vec![TokenType::EOF])]
    #[case("+-/*", vec![TokenType::Plus, TokenType::Minus, TokenType::Slash, TokenType::Star, TokenType::EOF])]
    #[case("()", vec![TokenType::LeftParen, TokenType::RightParen, TokenType::EOF])]
    #[case("{}", vec![TokenType::LeftBrace, TokenType::RightBrace, TokenType::EOF])]
    #[case(";.,", vec![TokenType::Semicolon, TokenType::Dot, TokenType::Comma, TokenType::EOF])]
    #[case("=", vec![TokenType::Equal, TokenType::EOF])]
    #[case("==", vec![TokenType::EqualEqual, TokenType::EOF])]
    #[case(">", vec![TokenType::Greater, TokenType::EOF])]
    #[case(">=", vec![TokenType::GreaterEqual, TokenType::EOF])]
    #[case("<", vec![TokenType::Less, TokenType::EOF])]
    #[case("<=", vec![TokenType::LessEqual, TokenType::EOF])]
    #[case("!", vec![TokenType::Bang, TokenType::EOF])]
    #[case("!=", vec![TokenType::BangEqual, TokenType::EOF])]
    #[case("   + -", vec![TokenType::Plus, TokenType::Minus, TokenType::EOF])]
    fn expected_values(#[case] input: String, #[case] expected: Vec<TokenType>) {
        let mut scanner = Scanner::new(&input);
        let mut output = vec![];
        loop {
            let current = scanner.scan().unwrap().token_type;
            output.push(current);
            if current == TokenType::EOF {
                break;
            }
        }
        assert_eq!(expected, output);
    }

    #[test]
    fn multiline() {
        let input = "+
        -"
        .to_string();
        let mut scanner = Scanner::new(&input);
        let token = scanner.scan().unwrap();
        assert_eq!(token.line, 1);
        assert_eq!(token.token_type, TokenType::Plus);
        let token = scanner.scan().unwrap();
        assert_eq!(token.line, 2);
        assert_eq!(token.token_type, TokenType::Minus);
        let token = scanner.scan().unwrap();
        assert_eq!(token.line, 2);
        assert_eq!(token.token_type, TokenType::EOF);
    }
}
