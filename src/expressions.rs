use crate::tokens::{Token, TokenLiteral};

pub type ChildExpression = Option<Box<Expression>>;

#[derive(Debug, Clone)]
pub enum Expression {
    Assign {
        name: Token,
        value: ChildExpression,
    },
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
    Logical {
        left: ChildExpression,
        operator: Token,
        right: ChildExpression,
    },
    Call {
        callee: ChildExpression,
        paren: Token,
        arguments: Vec<ChildExpression>,
    },
}

pub fn create_assignment(name: Token, value: ChildExpression) -> ChildExpression {
    Some(Box::new(Expression::Assign { name, value }))
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

pub fn create_logical(left: ChildExpression, operator: Token, right: ChildExpression) -> ChildExpression {
    Some(Box::new(Expression::Logical { left, operator, right }))
}

pub fn create_call(callee: ChildExpression, paren: Token, arguments: Vec<ChildExpression>) -> ChildExpression {
    Some(Box::new(Expression::Call { callee, paren, arguments }))
}
