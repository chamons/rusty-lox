use anyhow::{Context, Result};
use wasm_encoder::*;

use crate::parser::{ChildExpression, ChildStatement, Expression, Statement, Token, TokenLiteral};

pub struct Compiler {
    module: Module,
    start: Function,
    types: TypeSection,
    imports: ImportSection,
    functions: FunctionSection,
}

impl Compiler {
    pub fn init() -> Self {
        Compiler {
            module: Module::new(),
            start: Function::new(vec![]),
            types: TypeSection::new(),
            imports: ImportSection::new(),
            functions: FunctionSection::new(),
        }
    }

    pub fn compile(&mut self, statements: &Vec<ChildStatement>) -> Result<Vec<u8>> {
        self.initialize_compile();

        self.resolve_list_of_statements(statements)?;

        self.finalize_compile()
    }

    fn initialize_compile(&mut self) {
        self.imports.import("imports", Some("clock_func"), EntityType::Function(0));
        self.types.function(vec![], vec![ValType::F64]);

        self.imports.import("imports", Some("log_str"), EntityType::Function(1));
        self.types.function(vec![ValType::I32, ValType::I32], vec![]);
    }

    fn finalize_compile(&mut self) -> Result<Vec<u8>> {
        self.write_sections();

        let wasm_bytes = self.generate_binary();
        // std::fs::write("/Users/donblas/tmp/mine.wasm", &wasm_bytes)?;

        let mut validator = wasmparser::Validator::new();
        validator.validate_all(&wasm_bytes)?;

        Ok(wasm_bytes.to_vec())
    }

    fn generate_binary(&mut self) -> Vec<u8> {
        let final_module = std::mem::replace(&mut self.module, Module::new());
        final_module.finish()
    }

    fn write_sections(&mut self) {
        // Section Order - This ordering matters, though we don't use every section today, they are left for future use (and ordering)

        // typesec
        self.module.section(&self.types);
        // importsec
        self.module.section(&self.imports);
        // funcsec
        self.module.section(&self.functions);
        // tablesec
        // memsec
        // globalsec
        // exportsec
        // startsec
        // elemsec
        // datacountsec
        // codesec
        // datasec
    }

    fn resolve_local(&mut self, expr: &ChildExpression, name: &Token) -> Result<()> {
        Ok(())
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn resolve_list_of_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn resolve_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn resolve_statement(&mut self, node: &ChildStatement) -> Result<()> {
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

    fn resolve_return_statement(&mut self, value: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn resolve_while_statement(&mut self, condition: &ChildExpression, body: &ChildStatement) -> Result<()> {
        Ok(())
    }

    fn resolve_conditional_statement(&mut self, condition: &ChildExpression, then_branch: &ChildStatement, else_branch: &Option<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn resolve_function_declaration(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn resolve_expression(&mut self, node: &ChildExpression) -> Result<()> {
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

    fn resolve_literal(&mut self, literal: &TokenLiteral) -> Result<()> {
        Ok(())
    }

    fn resolve_logical(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn resolve_call_expression(&mut self, callee: &ChildExpression, arguments: &Vec<ChildExpression>) -> Result<()> {
        Ok(())
    }

    fn resolve_binary(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn resolve_assign_expression(&mut self, name: &Token, value: &ChildExpression, node: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn resolve_variable_expression(&mut self, name: &Token, node: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn resolve_variable_statement(&mut self, name: &Token, initializer: &ChildExpression) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::utils::BackEnd;

    use super::*;

    fn execute(script: &str) -> Result<String, String> {
        let script = &format!("print {};", script);

        let value = Rc::new(RefCell::new(None));
        let value_interp = Rc::clone(&value);

        let mut back_end = super::super::CompilerBackEnd::init(Box::new(move |p: &str| {
            value_interp.borrow_mut().replace(p.to_string());
        }));
        back_end.execute_script(script).map_err(|e| e.to_string())?;

        let value = value.borrow().clone().unwrap_or("".to_string());
        Ok(value)
    }

    #[test]
    fn single_values() {
        assert_eq!(Ok("42.0".to_string()), execute("42"));
    }
}
