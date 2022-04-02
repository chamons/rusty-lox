use crate::parser::{ChildExpression, ChildStatement, Expression, Statement, Token, TokenLiteral};

pub struct Compiler {}

impl Compiler {
    pub fn init() -> Self {
        Compiler {}
    }

    fn resolve_local(&mut self, expr: &ChildExpression, name: &Token) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_list_of_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn resolve_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_statement(&mut self, node: &ChildStatement) -> Result<(), &'static str> {
        if let Some(node) = node {
            match &**node {
                Statement::Block { statements } => self.resolve_statements(statements),
                Statement::Variable { name, initializer } => self.resolve_variable_statement(name, initializer),
                Statement::Function { body, name, params } => self.resolve_function_declaration(name, params, body),
                Statement::Expression { expression } => self.resolve_expression(expression),
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                } => self.resolve_conditional_statement(condition, then_branch, else_branch),
                Statement::Print { expression } => self.resolve_expression(expression),
                Statement::Return { value } => self.resolve_return_statement(value),
                Statement::While { condition, body } => self.resolve_while_statement(condition, body),
            }
        } else {
            Ok(())
        }
    }

    fn resolve_return_statement(&mut self, value: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_while_statement(&mut self, condition: &ChildExpression, body: &ChildStatement) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_conditional_statement(
        &mut self,
        condition: &ChildExpression,
        then_branch: &ChildStatement,
        else_branch: &Option<ChildStatement>,
    ) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_function_declaration(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_expression(&mut self, node: &ChildExpression) -> Result<(), &'static str> {
        if let Some(n) = node {
            match &**n {
                Expression::Variable { name } => self.resolve_variable_expression(name, node),
                Expression::Assign { name, value } => self.resolve_assign_expression(name, value, node),
                Expression::Binary { left, right, .. } => self.resolve_binary(left, right),
                Expression::Call { callee, arguments } => self.resolve_call_expression(callee, arguments),
                Expression::Grouping { expression } => self.resolve_expression(expression),
                Expression::Literal { value } => self.resolve_literal(value),
                Expression::Logical { left, right, .. } => self.resolve_logical(left, right),
                Expression::Unary { right, .. } => self.resolve_expression(right),
            }
        } else {
            Ok(())
        }
    }

    fn resolve_literal(&mut self, literal: &TokenLiteral) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_logical(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_call_expression(&mut self, callee: &ChildExpression, arguments: &Vec<ChildExpression>) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_binary(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_assign_expression(&mut self, name: &Token, value: &ChildExpression, node: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_variable_expression(&mut self, name: &Token, node: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    fn resolve_variable_statement(&mut self, name: &Token, initializer: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }
}
