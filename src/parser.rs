use crate::expressions::*;
use crate::tokens::{Token, TokenKind, TokenLiteral};

struct Parser {
    tokens: Vec<Token>,
    current: u32,
}

impl Parser {
    fn expression(&mut self) -> ChildExpression {
        return self.equality();
    }

    fn equality(&mut self) -> ChildExpression {
        let mut expr = self.comparison();

        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = create_binary(expr, operator, right);
        }
        expr
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        self.match_tokens(&[kind])
    }

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.at_end() {
            false
        } else {
            self.peek().kind == *kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current as usize).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get((self.current - 1) as usize).unwrap()
    }

    fn comparison(&mut self) -> ChildExpression {
        let mut expr = self.term();
        while self.match_tokens(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = create_binary(expr, operator, right);
        }
        expr
    }

    fn term(&mut self) -> ChildExpression {
        let mut expr = self.factor();
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = create_binary(expr, operator, right);
        }
        expr
    }

    fn factor(&mut self) -> ChildExpression {
        let mut expr = self.unary();

        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = create_binary(expr, operator, right);
        }
        expr
    }

    fn unary(&mut self) -> ChildExpression {
        if self.match_tokens(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();
            create_unary(operator, right)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ChildExpression {
        if self.match_token(TokenKind::False) {
            create_literal(TokenLiteral::Boolean(false))
        } else if self.match_token(TokenKind::True) {
            create_literal(TokenLiteral::Boolean(true))
        } else if self.match_token(TokenKind::Nil) {
            create_literal(TokenLiteral::Nil)
        } else if self.match_tokens(&[TokenKind::Number, TokenKind::String]) {
            create_literal(self.previous().literal.clone())
        } else {
            if self.match_token(TokenKind::LeftParen) {
                let expr = self.expression();
                self.consume(TokenKind::RightParen, "Expect ')' after expression");
                create_grouping(expr)
            } else {
                // Fix
                panic!();
            }
        }
    }

    // https://craftinginterpreters.com/parsing-expressions.html#entering-panic-mode
    fn consume(&mut self, kind: TokenKind, message: &str) {
        //Fix
    }
}
