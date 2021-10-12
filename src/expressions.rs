use std::fmt::Write;

use crate::tokens::{Token, TokenLiteral};

pub type ChildExpression = Option<Box<Expression>>;

pub enum Expression {
    Binary {
        left: ChildExpression,
        operator: Token,
        right: ChildExpression,
    },
    Grouping {
        expression: ChildExpression,
    },
    Literal {
        value: TokenLiteral,
    },
    Unary {
        operator: Token,
        right: ChildExpression,
    },
}

pub fn create_binary(left: ChildExpression, operator: Token, right: ChildExpression) -> ChildExpression {
    Some(Box::new(Expression::Binary { left, operator, right }))
}

pub fn create_grouping(expression: ChildExpression) -> ChildExpression {
    Some(Box::new(Expression::Grouping { expression }))
}

pub fn create_literal(value: TokenLiteral) -> ChildExpression {
    Some(Box::new(Expression::Literal { value }))
}

pub fn create_unary(operator: Token, right: ChildExpression) -> ChildExpression {
    Some(Box::new(Expression::Unary { operator, right }))
}

pub fn print_tree(root: ChildExpression, buf: &mut String) {
    if let Some(root) = root {
        match *root {
            Expression::Binary { left, operator, right } => {
                print_tree(left, buf);
                write!(buf, " {:?} ", operator.kind).unwrap();
                print_tree(right, buf);
            }
            Expression::Grouping { expression } => {
                write!(buf, "(").unwrap();
                print_tree(expression, buf);
                write!(buf, ")").unwrap();
            }
            Expression::Literal { value } => match value {
                TokenLiteral::Null => write!(buf, "nil").unwrap(),
                TokenLiteral::String(v) => write!(buf, "{}", v).unwrap(),
                TokenLiteral::Number(v) => write!(buf, "{}", v).unwrap(),
            },
            Expression::Unary { operator, right } => {
                write!(buf, "{:?} ", operator.kind).unwrap();
                print_tree(right, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::TokenKind;

    fn create_test_token(kind: TokenKind) -> Token {
        Token::init(kind, "", TokenLiteral::Null, 0)
    }

    #[test]
    pub fn print_complex_tree() {
        let root = create_unary(
            create_test_token(TokenKind::Minus),
            create_grouping(create_binary(
                create_literal(TokenLiteral::Number(1.0)),
                create_test_token(TokenKind::Plus),
                create_literal(TokenLiteral::Number(2.0)),
            )),
        );
        let mut buf = String::new();
        print_tree(root, &mut buf);
        assert_eq!("Minus (1 Plus 2)", buf);
    }
}
