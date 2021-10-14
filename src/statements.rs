use crate::{expressions::*, tokens::Token};

pub type ChildStatement = Option<Box<Statement>>;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression {
        expression: ChildExpression,
    },
    Print {
        expression: ChildExpression,
    },
    Variable {
        name: Token,
        initializer: ChildExpression,
    },
    Block {
        statements: Vec<ChildStatement>,
    },
    If {
        condition: ChildExpression,
        then_branch: ChildStatement,
        else_branch: Option<ChildStatement>,
    },
}

pub fn create_expression_statement(expression: ChildExpression) -> ChildStatement {
    Some(Box::new(Statement::Expression { expression }))
}

pub fn create_print_statement(expression: ChildExpression) -> ChildStatement {
    Some(Box::new(Statement::Print { expression }))
}

pub fn create_variable_statement(name: Token, initializer: ChildExpression) -> ChildStatement {
    Some(Box::new(Statement::Variable { name, initializer }))
}

pub fn create_block_statement(statements: Vec<ChildStatement>) -> ChildStatement {
    Some(Box::new(Statement::Block { statements }))
}

pub fn create_if_statement(condition: ChildExpression, then_branch: ChildStatement, else_branch: Option<ChildStatement>) -> ChildStatement {
    Some(Box::new(Statement::If {
        condition,
        then_branch,
        else_branch,
    }))
}
