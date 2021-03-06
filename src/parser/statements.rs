use super::{expressions::*, tokens::Token};

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
    While {
        condition: ChildExpression,
        body: ChildStatement,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<ChildStatement>,
    },
    Return {
        value: ChildExpression,
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

pub fn create_while_statement(condition: ChildExpression, body: ChildStatement) -> ChildStatement {
    Some(Box::new(Statement::While { condition, body }))
}

pub fn create_function_statement(name: Token, params: Vec<Token>, body: Vec<ChildStatement>) -> ChildStatement {
    Some(Box::new(Statement::Function { name, params, body }))
}

pub fn create_return_statement(value: ChildExpression) -> ChildStatement {
    Some(Box::new(Statement::Return { value }))
}
