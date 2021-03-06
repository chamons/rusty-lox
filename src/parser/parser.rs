use super::expressions::*;
use super::statements::*;
use super::tokens::{Token, TokenKind, TokenLiteral};

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

    pub fn parse_single_expression(&mut self) -> Result<ChildExpression, &'static str> {
        self.expression()
    }

    pub fn reset_position(&mut self) {
        self.current = 0;
    }

    fn declaration(&mut self) -> Result<ChildStatement, &'static str> {
        let result = if self.match_token(TokenKind::Fun) {
            self.function_declaration()
        } else if self.match_token(TokenKind::Var) {
            self.variable_declaration()
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn function_declaration(&mut self) -> Result<ChildStatement, &'static str> {
        let name = self.consume(TokenKind::Identifier, "Expected identifier")?.clone();
        self.consume(TokenKind::LeftParen, "Expect '(' after identifier.")?;
        let mut params = vec![];
        if !self.check(TokenKind::RightParen) {
            loop {
                if params.len() > 255 {
                    return Err("Can't have more than 255 parameters.");
                }
                params.push(self.consume(TokenKind::Identifier, "Expect parameter name.")?.clone());
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;
        let body = self.block()?;
        Ok(create_function_statement(name, params, body))
    }

    fn variable_declaration(&mut self) -> Result<ChildStatement, &'static str> {
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?.clone();

        let initializer = if self.match_token(TokenKind::Equal) { self.expression()? } else { None };
        self.consume(TokenKind::Semicolon, "Expect ';' after variable declaration.")?;

        Ok(create_variable_statement(name, initializer))
    }

    fn statement(&mut self) -> Result<ChildStatement, &'static str> {
        if self.match_token(TokenKind::For) {
            self.for_statement()
        } else if self.match_token(TokenKind::If) {
            self.if_statement()
        } else if self.match_token(TokenKind::Print) {
            self.print_statement()
        } else if self.match_token(TokenKind::Return) {
            self.return_statement()
        } else if self.match_token(TokenKind::While) {
            self.while_statement()
        } else if self.match_token(TokenKind::LeftBrace) {
            Ok(create_block_statement(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> Result<Vec<ChildStatement>, &'static str> {
        let mut statements = vec![];
        while !self.check(TokenKind::RightBrace) && !self.at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn for_statement(&mut self) -> Result<ChildStatement, &'static str> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_token(TokenKind::Semicolon) {
            None
        } else if self.match_token(TokenKind::Var) {
            self.variable_declaration()?
        } else {
            self.expression_statement()?
        };

        let condition = if !self.check(TokenKind::Semicolon) { self.expression()? } else { None };
        self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenKind::RightParen) { self.expression()? } else { None };
        self.consume(TokenKind::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        // Sprinkle some sugar on it...
        if let Some(increment) = increment {
            body = create_block_statement(vec![body, create_expression_statement(Some(increment))]);
        }
        let condition = condition.or_else(|| create_literal(TokenLiteral::Boolean(true)));

        body = create_while_statement(condition, body);
        if let Some(initializer) = initializer {
            body = create_block_statement(vec![Some(initializer), body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<ChildStatement, &'static str> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(TokenKind::Else) { Some(self.statement()?) } else { None };

        Ok(create_if_statement(condition, then_branch, else_branch))
    }

    fn return_statement(&mut self) -> Result<ChildStatement, &'static str> {
        let value = if !self.check(TokenKind::Semicolon) { self.expression()? } else { None };
        self.consume(TokenKind::Semicolon, "Expect ';' after return value")?;
        Ok(create_return_statement(value))
    }

    fn while_statement(&mut self) -> Result<ChildStatement, &'static str> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(create_while_statement(condition, body))
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
        self.assignment()
    }

    fn assignment(&mut self) -> Result<ChildExpression, &'static str> {
        let expr = self.or()?;

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

    fn or(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.and()?;

        while self.match_token(TokenKind::Or) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = create_logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.equality()?;

        while self.match_token(TokenKind::And) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = create_logical(expr, operator, right);
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
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn at_end(&self) -> bool {
        self.peek().kind == TokenKind::EndOfFile
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
            self.call()
        }
    }

    fn call(&mut self) -> Result<ChildExpression, &'static str> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: ChildExpression) -> Result<ChildExpression, &'static str> {
        let mut arguments = vec![];

        if !self.check(TokenKind::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err("Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;

        Ok(create_call(callee, arguments))
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
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(message)
        }
    }

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
    use super::super::Scanner;
    use super::*;

    fn parses_without_errors(script: &str) {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        assert!(parser.parse().is_ok());
    }

    fn parses_with_errors(script: &str) {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn parse_simple_expression() {
        parses_without_errors("1 + 2;");
        parses_without_errors("-(1 + 2) * 4 / (4 + 1 - 2.3);");
    }

    #[test]
    fn parse_mismatched_braces() {
        parses_with_errors("(;");
        parses_with_errors("-(1 + 2;");
        // TODO - Need statement support to detect? We seem to eager
        // parses_with_errors("2)");
        // parses_with_errors(")");
    }

    #[test]
    fn parse_leading_op() {
        parses_with_errors("+ 2;");
    }

    #[test]
    fn parse_equality_and_comparisons() {
        parses_without_errors("2 == 3;");
        parses_without_errors("2 != 3;");
        parses_without_errors("2 <= 3;");
        parses_without_errors("2 < 3;");
        parses_without_errors("2 >= 3;");
        parses_without_errors("2 > 3;");
    }

    #[test]
    fn parse_block() {
        parses_without_errors(
            "{
            2 == 3;
            2 == 3;
        }",
        );
        parses_with_errors(
            "{
            2 == 3;
            2 == 3;",
        );
    }

    #[test]
    fn parse_conditional() {
        parses_without_errors(
            "if (true == false) {
                2 == 3;
            }",
        );
        parses_with_errors(
            "if (true == false) {
                2 == 3;
            ",
        );
        parses_with_errors(
            "if (true == false)
                2 == 3;
            }",
        );
    }

    #[test]
    fn parse_logical_conditional() {
        parses_without_errors(
            "if (true and false) {
                2 == 3;
            }",
        );
        parses_without_errors(
            "if (true or false) {
                2 == 3;
            }",
        );
        parses_without_errors(
            "if (true or false and 1 == 2 or (true and false or true)) {
                2 == 3;
            }",
        );
        parses_with_errors(
            "if (true and) {
                2 == 3;
            ",
        );
        parses_with_errors(
            "if (or false)
                2 == 3;
            }",
        );
    }

    #[test]
    fn parse_while() {
        parses_without_errors(
            "while (true) {
                2 == 3;
            }",
        );
        parses_without_errors(
            "while (true and false) {
                2 == 3;
            }",
        );
        parses_with_errors(
            "while (true {
                2 == 3;
            }",
        );
        parses_with_errors(
            "while true) {
                2 == 3;
            }",
        );
        parses_with_errors("while (true)");
    }

    #[test]
    fn parse_for() {
        parses_without_errors(
            "var i;
            for (i = 0; i < 10; i = i + 1) {
                print i;
            }",
        );
        parses_without_errors(
            "for (var i = 0; i < 10; i = i + 1) {
                print i;
            }",
        );
        parses_without_errors(
            "for (;;) {
                print i;
            }",
        );
        parses_with_errors(
            "for (var i = 0; i = i + 1) {
                print i;
            }",
        );
        parses_with_errors(
            "for (i < 10; i = i + 1) {
                print i;
            }",
        );
        parses_with_errors(
            "for (var i = 0; i < 10) {
                print i;
            }",
        );
        parses_with_errors(
            "for var i = 0; i < 10; i = i + 1) {
                print i;
            }",
        );
        parses_with_errors(
            "for (var i = 0; i < 10; i = i + 1 {
                print i;
            }",
        );
    }

    #[test]
    fn parse_call() {
        parses_without_errors("foo();");
        parses_without_errors("foo(1, true, nil);");
        parses_with_errors("foo(1, true, nil;");
        parses_with_errors("foo 1, true, nil);");
        parses_without_errors("if (n > 1) count(n - 1);");
    }

    // No more than 255 arguments
    #[test]
    fn max_parse_call() {
        let mut script = "foo(".to_string();
        for _ in 0..255 {
            script.push_str("true,");
        }
        script.remove(script.len() - 1);
        script.push_str(");");
        parses_without_errors(&script);

        let mut script = "foo(".to_string();
        for _ in 0..256 {
            script.push_str("true,");
        }
        script.remove(script.len() - 1);
        script.push_str(");");
        parses_with_errors(&script);
    }

    #[test]
    fn function_declare() {
        parses_without_errors("fun t() { print true; }");
    }

    #[test]
    fn return_declare() {
        parses_without_errors("fun t() { return true; }");
        parses_without_errors("fun t() { return; }");
        parses_with_errors("fun t() { return 42.0 }");
        parses_with_errors("fun t() { return }");
    }
}
