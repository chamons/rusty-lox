use std::collections::HashMap;

use crate::interpreter;
use crate::interpreter::Interpreter;

use super::expressions::*;
use super::statements::*;
use super::tokens::Token;

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: Interpreter,
}

impl Resolver {
    pub fn init(interpreter: Interpreter) -> Self {
        Resolver { scopes: vec![], interpreter }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexme.to_string(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexme.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &ChildExpression, name: &Token) -> Result<(), &'static str> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexme) {
                self.interpreter.resolve(expr, i)?;
            }
        }
        Ok(())
    }

    pub fn resolve_block_statement(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    pub fn resolve_statement(&mut self, node: &ChildStatement) -> Result<(), &'static str> {
        if let Some(node) = node {
            match &**node {
                Statement::Block { statements } => self.resolve_block_statement(statements),
                Statement::Expression { expression } => self.resolve_expression_statement(expression),
                Statement::Variable { name, initializer } => self.resolve_variable_statement(name, initializer),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    pub fn resolve_expression(&mut self, node: &ChildExpression) -> Result<(), &'static str> {
        if let Some(n) = node {
            match &**n {
                Expression::Variable { name } => self.resolve_variable_expression(name, node),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    pub fn resolve_variable_expression(&mut self, name: &Token, node: &ChildExpression) -> Result<(), &'static str> {
        if let Some(scope) = self.scopes.last() {
            let variable = scope.get(&name.lexme);
            if variable.is_none() || *variable.unwrap() == false {
                return Err("Can't read local variable in its own initializer.");
            }
        }
        self.resolve_local(node, name);
        Ok(())
    }

    pub fn resolve_expression_statement(&mut self, expression: &ChildExpression) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn resolve_variable_statement(&mut self, name: &Token, initializer: &ChildExpression) -> Result<(), &'static str> {
        self.declare(name);
        if initializer.is_some() {
            self.resolve_expression(initializer)?;
        }
        self.define(name);
        Ok(())
    }
}
