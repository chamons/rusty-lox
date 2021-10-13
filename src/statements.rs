use crate::{expressions::*, tokens::Token};

pub type ChildStatement = Option<Box<Statements>>;

#[derive(Debug, Clone)]
pub enum Statements {
    Expression { expression: ChildExpression },
    Print { expression: ChildExpression },
    Variable { name: Token, initializer: Option<ChildExpression> },
}

pub fn create_expression_statement(expression: ChildExpression) -> ChildStatement {
    Some(Box::new(Statements::Expression { expression }))
}

pub fn create_print_statement(expression: ChildExpression) -> ChildStatement {
    Some(Box::new(Statements::Print { expression }))
}

pub fn create_variable_statement(name: Token, initializer: Option<ChildExpression>) -> ChildStatement {
    Some(Box::new(Statements::Variable { name, initializer }))
}
