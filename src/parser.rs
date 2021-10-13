use crate::expressions::*;
use crate::statements::*;
use crate::tokens::{Token, TokenKind, TokenLiteral};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: u32,
}

impl<'a> Parser<'a> {
    pub fn init(tokens: &'a Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<ChildStatement>, &'static str> {
        let mut statements = vec![];
        while !self.at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<ChildStatement, &'static str> {
        let result = if self.match_token(TokenKind::Var) {
            self.variable_declaration()
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn variable_declaration(&mut self) -> Result<ChildStatement, &'static str> {
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?.clone();

        let initializer = if self.match_token(TokenKind::Equal) { Some(self.expression()?) } else { None };
        self.consume(TokenKind::Semicolon, "Expect ';' after variable declaration.")?;

        Ok(create_variable_statement(name.clone(), initializer))
    }

    fn statement(&mut self) -> Result<ChildStatement, &'static str> {
        if self.match_token(TokenKind::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<ChildStatement, &'static str> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(create_print_statement(value))
    }

    fn expression_statement(&mut self) -> Result<ChildStatement, &'static str> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(create_expression_statement(value))
    }

    fn expression(&mut self) -> Result<ChildExpression, &'static str> {
        Ok(self.assignment()?)
    }

    fn assignment(&mut self) -> Result<ChildExpression, &'static str> {
        let expr = self.equality()?;

        if self.match_token(TokenKind::Equal) {
            let value = self.assignment()?;
            return match expr {
                Some(v) => match *v {
                    Expression::Variable { name } => Ok(create_assignment(name, value)),
                    _ => Err("Invalid assignment target."),
                },
                _ => Err("Invalid assignment target."),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = create_binary(expr, operator, right);
        }
        Ok(expr)
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

    fn comparison(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.term()?;
        while self.match_tokens(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = create_binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = create_binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = create_binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<ChildExpression, &'static str> {
        if self.match_tokens(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(create_unary(operator, right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<ChildExpression, &'static str> {
        if self.match_token(TokenKind::False) {
            Ok(create_literal(TokenLiteral::Boolean(false)))
        } else if self.match_token(TokenKind::True) {
            Ok(create_literal(TokenLiteral::Boolean(true)))
        } else if self.match_token(TokenKind::Nil) {
            Ok(create_literal(TokenLiteral::Nil))
        } else if self.match_tokens(&[TokenKind::Number, TokenKind::String]) {
            Ok(create_literal(self.previous().literal.clone()))
        } else {
            if self.match_token(TokenKind::Identifier) {
                Ok(create_variable(self.previous().clone()))
            } else if self.match_token(TokenKind::LeftParen) {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expect ')' after expression")?;
                Ok(create_grouping(expr))
            } else {
                Err("Expect expression.")
            }
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &'static str) -> Result<&Token, &'static str> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(message)
        }
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Scanner;

    fn parses_without_errors(script: &str) {
        let mut scanner = Scanner::init(&format!("{};", script));
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        assert!(parser.parse().is_ok());
    }

    fn parses_with_errors(script: &str) {
        let mut scanner = Scanner::init(&format!("{};", script));
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn parse_simple_expression() {
        parses_without_errors("1 + 2");
        parses_without_errors("-(1 + 2) * 4 / (4 + 1 - 2.3)");
    }

    #[test]
    fn parse_mismatched_braces() {
        parses_with_errors("(");
        parses_with_errors("-(1 + 2");
        // TODO - Need statement support to detect? We seem to eager
        // parses_with_errors("2)");
        // parses_with_errors(")");
    }

    #[test]
    fn parse_leading_op() {
        parses_with_errors("+ 2");
    }

    #[test]
    fn parse_equality_and_comparisions() {
        parses_without_errors("2 == 3");
        parses_without_errors("2 != 3");
        parses_without_errors("2 <= 3");
        parses_without_errors("2 < 3");
        parses_without_errors("2 >= 3");
        parses_without_errors("2 > 3");
    }
}
