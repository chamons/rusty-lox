use super::{
    source::Source,
    token::{Token, TokenType},
};

pub struct Scanner<'a> {
    source: Source<'a>,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: Source::new(source),
            line: 1,
        }
    }

    pub fn scan(&mut self) -> eyre::Result<Token> {
        self.skip_whitespace();

        let c = match self.advance() {
            Some(c) => c,
            None => {
                return self.token(TokenType::Eof);
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
        self.source.next()
    }

    fn match_character(&mut self, expected: char) -> bool {
        match self.source.peek() {
            Some(c) => {
                if c == expected {
                    _ = self.advance();
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
            match self.source.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') => {
                    if self.source.peek_two() == Some('/') {
                        loop {
                            match self.source.peek() {
                                Some('\n') | None => {
                                    break;
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                        }
                    } else {
                        return;
                    }
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
    #[case("", vec![TokenType::Eof])]
    #[case("+-/*", vec![TokenType::Plus, TokenType::Minus, TokenType::Slash, TokenType::Star, TokenType::Eof])]
    #[case("()", vec![TokenType::LeftParen, TokenType::RightParen, TokenType::Eof])]
    #[case("{}", vec![TokenType::LeftBrace, TokenType::RightBrace, TokenType::Eof])]
    #[case(";.,", vec![TokenType::Semicolon, TokenType::Dot, TokenType::Comma, TokenType::Eof])]
    #[case("=", vec![TokenType::Equal, TokenType::Eof])]
    #[case("==", vec![TokenType::EqualEqual, TokenType::Eof])]
    #[case(">", vec![TokenType::Greater, TokenType::Eof])]
    #[case(">=", vec![TokenType::GreaterEqual, TokenType::Eof])]
    #[case("<", vec![TokenType::Less, TokenType::Eof])]
    #[case("<=", vec![TokenType::LessEqual, TokenType::Eof])]
    #[case("!", vec![TokenType::Bang, TokenType::Eof])]
    #[case("!=", vec![TokenType::BangEqual, TokenType::Eof])]
    #[case("   + -", vec![TokenType::Plus, TokenType::Minus, TokenType::Eof])]
    #[case("+ // This is a comment", vec![TokenType::Plus, TokenType::Eof])]
    fn expected_values(#[case] input: String, #[case] expected: Vec<TokenType>) {
        let mut scanner = Scanner::new(&input);
        let mut output = vec![];
        loop {
            let current = scanner.scan().unwrap().token_type;
            output.push(current);
            if current == TokenType::Eof {
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
        assert_eq!(token.token_type, TokenType::Eof);
    }
}
