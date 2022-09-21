use super::tokens::{Token, TokenLiteral};

pub type ChildExpression = Option<Box<Expression>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Expression {
    Assign {
        name: Token,
        value: ChildExpression,
        line: u32,
    },
    Binary {
        left: ChildExpression,
        operator: Token,
        right: ChildExpression,
        line: u32,
    },
    Grouping {
        expression: ChildExpression,
        line: u32,
    },
    Literal {
        value: TokenLiteral,
        line: u32,
    },
    Unary {
        operator: Token,
        right: ChildExpression,
        line: u32,
    },
    Variable {
        name: Token,
        line: u32,
    },
    Logical {
        left: ChildExpression,
        operator: Token,
        right: ChildExpression,
        line: u32,
    },
    Call {
        callee: ChildExpression,
        arguments: Vec<ChildExpression>,
        line: u32,
    },
}

pub fn create_assignment(name: Token, value: ChildExpression, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Assign { name, value, line }))
}

pub fn create_binary(left: ChildExpression, operator: Token, right: ChildExpression, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Binary { left, operator, right, line }))
}

pub fn create_grouping(expression: ChildExpression, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Grouping { expression, line }))
}

pub fn create_literal(value: TokenLiteral, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Literal { value, line }))
}

pub fn create_unary(operator: Token, right: ChildExpression, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Unary { operator, right, line }))
}

pub fn create_variable(name: Token, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Variable { name, line }))
}

pub fn create_logical(left: ChildExpression, operator: Token, right: ChildExpression, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Logical { left, operator, right, line }))
}

pub fn create_call(callee: ChildExpression, arguments: Vec<ChildExpression>, line: u32) -> ChildExpression {
    Some(Box::new(Expression::Call { callee, arguments, line }))
}
