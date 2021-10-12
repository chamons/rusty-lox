use std::fmt::{Debug, Display};

#[allow(dead_code)]
#[derive(Debug)]
enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug)]
enum TokenLiteral {
    Null,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    lexme: String,
    literal: TokenLiteral,
    line: u32,
}

impl Token {
    pub fn init(kind: TokenKind, lexme: &str, literal: TokenLiteral, line: u32) -> Self {
        Token {
            kind,
            lexme: lexme.to_string(),
            literal,
            line,
        }
    }
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

#[derive(Debug)]
struct Scanner {
    source: String,
    tokens: Vec<Token>,
    errors: Vec<ScannerError>,
    start: u32,
    current: u32,
    line: u32,
}

impl Scanner {
    pub fn init(source: &str) -> Self {
        Scanner {
            source: source.to_string(),
            tokens: vec![],
            errors: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> (&Vec<Token>, &Vec<ScannerError>) {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::init(TokenKind::EOF, "", TokenLiteral::Null, self.line));
        (&self.tokens, &self.errors)
    }

    fn scan_token(&mut self) {
        let token = self.advance();
        match token {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                if self.match_token('=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_token('=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_token('=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_token('=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            '/' => {
                if self.match_token('/') {
                    while self.peek() != '\n' && !self.at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            _ => self.error(&format!("Unexpected character: {}", token)),
        }
    }

    fn peek(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.current_char()
        }
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.at_end() || self.current_char() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.add_token_with_value(kind, TokenLiteral::Null)
    }

    fn add_token_with_value(&mut self, kind: TokenKind, literal: TokenLiteral) {
        let text = self.source[self.start as usize..self.current as usize].to_string();
        self.tokens.push(Token::init(kind, &text, literal, self.line));
    }

    fn current_char(&self) -> char {
        self.source.as_bytes()[self.current as usize] as char
    }

    fn advance(&mut self) -> char {
        let value = self.current_char();
        self.current += 1;
        value as char
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len() as u32
    }

    fn error(&mut self, message: &str) {
        self.report("", message)
    }

    fn report(&mut self, location: &str, message: &str) {
        self.errors.push(ScannerError::init(self.line, location, message));
    }
}

pub fn run(script: &str) -> Vec<ScannerError> {
    let mut scanner = Scanner::init(script);
    let (tokens, errors) = scanner.scan_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
    errors.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_no_errors(input: &str) {
        let errors = run(input);
        for error in &errors {
            println!("{}", error);
        }
        assert_eq!(0, errors.len(), "On input: '{}'", input);
    }

    #[test]
    pub fn single_token() {
        for c in ['(', ')', '{', '}', ',', '.', '-', '+', ';', '*', '!', '=', '<', '>', '/'] {
            input_no_errors(&format!("{}", c));
        }
    }

    #[test]
    pub fn multi_token() {
        for c in ["!=", "==", "<=", ">=", "//"] {
            input_no_errors(&format!("{}", c));
        }
    }

    #[test]
    pub fn comment() {
        input_no_errors("{}// Hello World");
    }

    #[test]
    pub fn spaces() {
        input_no_errors("{} ()  // Comment");
    }

    #[test]
    pub fn input_multiple_lines() {
        input_no_errors(
            "{}
()
// Comment",
        );
    }
}
