use crate::tokens::{Token, TokenLiteral};

pub type ChildExpression = Option<Box<Expression>>;

#[derive(Debug, Clone)]
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
    Variable {
        name: Token,
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

pub fn create_variable(name: Token) -> ChildExpression {
    Some(Box::new(Expression::Variable { name }))
}
