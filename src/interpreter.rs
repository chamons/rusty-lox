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
        match self.advance() {
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
            token @ _ => self.error(&format!("Unexpected character: {}", token)),
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.add_token_with_value(kind, TokenLiteral::Null)
    }

    fn add_token_with_value(&mut self, kind: TokenKind, literal: TokenLiteral) {
        let text = self.source[self.start as usize..self.current as usize].to_string();
        self.tokens.push(Token::init(kind, &text, literal, self.line));
    }

    fn advance(&mut self) -> char {
        let value = self.source.as_bytes()[self.current as usize];
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
