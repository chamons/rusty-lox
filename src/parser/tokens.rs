use std::{collections::HashMap, hash};

use super::utils::ScannerError;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TokenKind {
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TokenLiteral {
    Nil,
    String(String),
    Number(HashableFloat),
    Boolean(bool),
}

// https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
// We only Eq/Hash tokens for resolution of lines/variables, so it is completely
// safe to have two different NaN not equal
#[derive(Debug, Copy, Clone)]
pub struct HashableFloat(f64);

impl HashableFloat {
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl HashableFloat {
    fn key(&self) -> u64 {
        self.0.to_bits()
    }
}

impl hash::Hash for HashableFloat {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.key().hash(state)
    }
}

impl PartialEq for HashableFloat {
    fn eq(&self, other: &HashableFloat) -> bool {
        self.key() == other.key()
    }
}

impl Eq for HashableFloat {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexme: String,
    pub literal: TokenLiteral,
    pub line: u32,
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

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = {
        let mut m = HashMap::new();
        m.insert("and", TokenKind::And);
        m.insert("class", TokenKind::Class);
        m.insert("else", TokenKind::Else);
        m.insert("false", TokenKind::False);
        m.insert("for", TokenKind::For);
        m.insert("fun", TokenKind::Fun);
        m.insert("if", TokenKind::If);
        m.insert("nil", TokenKind::Nil);
        m.insert("or", TokenKind::Or);
        m.insert("print", TokenKind::Print);
        m.insert("return", TokenKind::Return);
        m.insert("super", TokenKind::Super);
        m.insert("this", TokenKind::This);
        m.insert("true", TokenKind::True);
        m.insert("var", TokenKind::Var);
        m.insert("while", TokenKind::While);
        m
    };
}

#[derive(Debug)]
pub struct Scanner {
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
        self.tokens.push(Token::init(TokenKind::EOF, "", TokenLiteral::Nil, self.line));
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
                if self.match_token('*') {
                    self.c_style_comment();
                } else if self.match_token('/') {
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
            '"' => self.string(),
            _ => {
                if token.is_ascii_digit() {
                    self.number()
                } else if token.is_ascii_alphabetic() {
                    self.identifier();
                } else {
                    self.error(&format!("Unexpected character: {}", token));
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
        let text = self.source[self.start as usize..self.current as usize].to_string();
        let kind = match KEYWORDS.get(&*text) {
            Some(keyword) => keyword.clone(),
            None => TokenKind::Identifier,
        };
        self.add_token(kind);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume .
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let text = self.source[self.start as usize..self.current as usize].to_string();
        match text.parse::<f64>() {
            Ok(v) => self.add_token_with_value(TokenKind::Number, TokenLiteral::Number(HashableFloat(v))),
            Err(_) => self.error(&format!("Invalid number: {}", text)),
        }
    }

    fn c_style_comment(&mut self) {
        while !(self.peek() == '*' && self.peek_next() == '/') && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            self.error("Unterminated comment.");
            return;
        }
        // Closing */
        self.advance();
        self.advance();
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            self.error("Unterminated string.");
            return;
        }

        // Closing "
        self.advance();

        self.add_token_with_value(
            TokenKind::String,
            TokenLiteral::String(self.source[self.start as usize + 1..self.current as usize - 1].to_string()),
        )
    }

    fn peek(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.current_char()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() as u32 {
            '\0'
        } else {
            self.source.as_bytes()[(self.current + 1) as usize] as char
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
        self.add_token_with_value(kind, TokenLiteral::Nil)
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

#[cfg(test)]
mod tests {
    use super::*;
    fn run(script: &str) -> (Vec<Token>, Vec<ScannerError>) {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        (tokens.clone(), errors.clone())
    }

    fn input_has_errors(input: &str) -> Vec<Token> {
        let (tokens, errors) = run(input);
        assert!(errors.len() > 0);
        tokens
    }

    fn input_no_errors(input: &str) -> Vec<Token> {
        let (tokens, errors) = run(input);
        for error in &errors {
            println!("{}", error);
        }
        assert_eq!(0, errors.len(), "On input: '{}'", input);
        tokens
    }

    fn matches_tokens(tokens: &Vec<Token>, expected: &[TokenKind]) {
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.kind, expected[i]);
        }
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
        let tokens = input_no_errors("{}// Hello World");
        matches_tokens(&tokens, &[TokenKind::LeftBrace, TokenKind::RightBrace, TokenKind::EOF]);
    }

    #[test]
    pub fn spaces() {
        let tokens = input_no_errors("{} ()  // Comment");
        matches_tokens(
            &tokens,
            &[
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::EOF,
            ],
        );
    }

    #[test]
    pub fn input_multiple_lines() {
        input_no_errors(
            "{}
()
// Comment",
        );
    }

    #[test]
    pub fn strings() {
        let tokens = input_no_errors("\"asdf fdsa {!/)\"");
        matches_tokens(&tokens, &[TokenKind::String, TokenKind::EOF]);
    }

    #[test]
    pub fn unterminated_strings() {
        input_has_errors("\"asdf fdsa");
    }

    #[test]
    pub fn numbers() {
        for n in &["1234", "12.34"] {
            let tokens = input_no_errors(n);
            matches_tokens(&tokens, &[TokenKind::Number, TokenKind::EOF]);
        }
    }

    #[test]
    pub fn trailing_point_numbers_are_separate() {
        let tokens = input_no_errors(".1234");
        matches_tokens(&tokens, &[TokenKind::Dot, TokenKind::Number, TokenKind::EOF]);

        let tokens = input_no_errors("1234.");
        matches_tokens(&tokens, &[TokenKind::Number, TokenKind::Dot, TokenKind::EOF]);
    }

    #[test]
    pub fn reserved_words() {
        for c in &[
            "and", "class", "else", "false", "for", "fun", "if", "nil", "or", "print", "return", "super", "this", "true", "var", "while",
        ] {
            input_no_errors(&format!("{}", c));
        }
    }

    #[test]
    pub fn identifier() {
        let tokens = input_no_errors("orchid");
        matches_tokens(&tokens, &[TokenKind::Identifier, TokenKind::EOF]);
    }

    #[test]
    pub fn reserved_words_inside_identifiers() {
        let tokens = input_no_errors("\"orchid\"");
        matches_tokens(&tokens, &[TokenKind::String, TokenKind::EOF]);
    }

    #[test]
    pub fn c_style_comment() {
        let tokens = input_no_errors("{}/* Hello World*/ ()");
        matches_tokens(
            &tokens,
            &[
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::EOF,
            ],
        );
    }

    #[test]
    pub fn c_style_comment_multiple_lines() {
        let tokens = input_no_errors(
            "{}/* Hello World

*/",
        );
        matches_tokens(&tokens, &[TokenKind::LeftBrace, TokenKind::RightBrace, TokenKind::EOF]);
    }

    #[test]
    pub fn c_style_comment_multiple_lines_not_terminated() {
        let tokens = input_has_errors(
            "{}/* Hello 
            
World",
        );
        matches_tokens(&tokens, &[TokenKind::LeftBrace, TokenKind::RightBrace, TokenKind::EOF]);
    }
}
