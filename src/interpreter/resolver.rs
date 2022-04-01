use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::Interpreter;

use crate::parser::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: Rc<RefCell<Interpreter>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn init(interpreter: &Rc<RefCell<Interpreter>>) -> Self {
        Resolver {
            scopes: vec![],
            interpreter: Rc::clone(interpreter),
            current_function: FunctionType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), &'static str> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexme.to_string()) {
                return Err("Already a variable with this name in this scope.");
            }
            scope.insert(name.lexme.to_string(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexme.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &ChildExpression, name: &Token) -> Result<(), &'static str> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexme) {
                self.interpreter.borrow_mut().resolve(expr, i)?;
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<ChildStatement>, kind: FunctionType) -> Result<(), &'static str> {
        let enclosing = self.current_function;
        self.current_function = kind;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_list_of_statements(body)?;
        self.end_scope();
        self.current_function = enclosing;
        Ok(())
    }

    fn resolve_list_of_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    pub fn resolve_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        self.begin_scope();
        self.resolve_list_of_statements(statements)?;
        self.end_scope();
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
        if self.current_function == FunctionType::None {
            return Err("Can't return from top-level code.");
        }
        self.resolve_expression(value)?;
        Ok(())
    }

    fn resolve_while_statement(&mut self, condition: &ChildExpression, body: &ChildStatement) -> Result<(), &'static str> {
        self.resolve_expression(condition)?;
        self.resolve_statement(body)?;
        Ok(())
    }

    fn resolve_conditional_statement(
        &mut self,
        condition: &ChildExpression,
        then_branch: &ChildStatement,
        else_branch: &Option<ChildStatement>,
    ) -> Result<(), &'static str> {
        self.resolve_expression(condition)?;
        self.resolve_statement(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_statement(else_branch)?;
        }
        Ok(())
    }

    fn resolve_function_declaration(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<(), &'static str> {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(params, body, FunctionType::Function)?;
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
                Expression::Literal { .. } => Ok(()),
                Expression::Logical { left, right, .. } => self.resolve_logical(left, right),
                Expression::Unary { right, .. } => self.resolve_expression(right),
            }
        } else {
            Ok(())
        }
    }

    fn resolve_logical(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<(), &'static str> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn resolve_call_expression(&mut self, callee: &ChildExpression, arguments: &Vec<ChildExpression>) -> Result<(), &'static str> {
        self.resolve_expression(callee)?;
        for arg in arguments {
            self.resolve_expression(arg)?;
        }
        Ok(())
    }

    fn resolve_binary(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<(), &'static str> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn resolve_assign_expression(&mut self, name: &Token, value: &ChildExpression, node: &ChildExpression) -> Result<(), &'static str> {
        self.resolve_expression(value)?;
        self.resolve_local(node, name)?;
        Ok(())
    }

    fn resolve_variable_expression(&mut self, name: &Token, node: &ChildExpression) -> Result<(), &'static str> {
        if let Some(scope) = self.scopes.last() {
            if scope.get(&name.lexme) == Some(&false) {
                return Err("Can't read local variable in its own initializer.");
            }
        }
        self.resolve_local(node, name)?;
        Ok(())
    }

    fn resolve_variable_statement(&mut self, name: &Token, initializer: &ChildExpression) -> Result<(), &'static str> {
        self.declare(name)?;
        if initializer.is_some() {
            self.resolve_expression(initializer)?;
        }
        self.define(name);
        Ok(())
    }
}
