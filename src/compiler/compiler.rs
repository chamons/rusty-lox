use std::collections::HashMap;

use anyhow::{anyhow, Result};
use wasm_encoder::*;

use crate::parser::{ChildExpression, ChildStatement, Expression, Statement, Token, TokenLiteral};

pub struct CompileContext<'a> {
    pub name: String,
    pub params: Vec<Token>,
    pub locals: Vec<TokenLiteral>,
    pub return_value: Option<Token>,
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> CompileContext<'a> {
    pub fn init(name: &str) -> Self {
        CompileContext {
            name: name.to_string(),
            params: vec![],
            locals: vec![],
            return_value: None,
            instructions: vec![],
        }
    }

    pub fn init_with_params(name: &str, params: &Vec<Token>) -> Self {
        CompileContext {
            name: name.to_string(),
            params: params.to_vec(),
            locals: vec![],
            return_value: None,
            instructions: vec![],
        }
    }
}

pub struct Compiler<'a> {
    module: Module,
    types: TypeSection,
    imports: ImportSection,
    functions: FunctionSection,
    codes: CodeSection,
    start_id: Option<u32>,

    current_index: u32, // How far are we into the various tables we're walking lockstep

    context: CompileContext<'a>,
    function_names: HashMap<String, u32>,
}

impl<'a> Compiler<'a> {
    pub fn init() -> Self {
        Compiler {
            module: Module::new(),
            types: TypeSection::new(),
            imports: ImportSection::new(),
            functions: FunctionSection::new(),
            codes: CodeSection::new(),
            start_id: None,
            context: CompileContext::init("start"),
            function_names: HashMap::new(),
            current_index: 0,
        }
    }

    pub fn compile(&mut self, statements: &Vec<ChildStatement>) -> Result<Vec<u8>> {
        self.initialize_compile();

        self.compile_list_of_statements(statements)?;

        self.finalize_compile()
    }

    fn initialize_compile(&mut self) {
        self.imports.import("imports", Some("clock"), EntityType::Function(self.current_index));
        self.types.function(vec![], vec![ValType::F64]);
        self.function_names.insert("clock".to_string(), self.current_index);
        self.current_index += 1;

        self.imports.import("imports", Some("log_str"), EntityType::Function(self.current_index));
        self.types.function(vec![ValType::I32, ValType::I32], vec![]);
        self.function_names.insert("log_str".to_string(), self.current_index);
        self.current_index += 1;

        self.imports.import("imports", Some("log_num"), EntityType::Function(self.current_index));
        self.types.function(vec![ValType::F64], vec![]);
        self.function_names.insert("log_num".to_string(), self.current_index);
        self.current_index += 1;
    }

    fn finalize_compile(&mut self) -> Result<Vec<u8>> {
        self.write_start();
        self.write_all_sections();

        let wasm_bytes = self.generate_binary();
        std::fs::write("/Users/donblas/tmp/mine.wasm", &wasm_bytes)?;

        let mut validator = wasmparser::Validator::new();
        validator.validate_all(&wasm_bytes)?;

        Ok(wasm_bytes.to_vec())
    }

    fn write_start(&mut self) {
        self.start_id = Some(self.current_index);
        self.finish_function();
    }

    fn generate_binary(&mut self) -> Vec<u8> {
        let final_module = std::mem::replace(&mut self.module, Module::new());
        final_module.finish()
    }

    fn write_all_sections(&mut self) {
        // Section Order - This ordering matters, though we don't use every section today, they are left for future use (and ordering)

        // typesec
        self.module.section(&self.types);
        // importsec
        self.module.section(&self.imports);
        // funcsec
        self.module.section(&self.functions);
        // tablesec
        // memsec
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            limits: Limits { min: 1, max: None },
        });
        self.module.section(&memories);
        // globalsec
        // exportsec
        // startsec
        if let Some(start_id) = &self.start_id {
            self.module.section(&StartSection { function_index: *start_id });
        }
        // elemsec
        // datacountsec
        // codesec
        self.module.section(&self.codes);
        // datasec
    }

    fn finish_function(&mut self) {
        // Finish off function with end instruction
        self.context.instructions.push(Instruction::End);

        self.functions.function(self.current_index);

        // Assume all params are f64
        self.types.function(self.context.params.iter().map(|_| ValType::F64), vec![]);

        let mut f = Function::new(vec![]);
        for instruction in &self.context.instructions {
            f.instruction(instruction.clone());
        }
        self.codes.function(&f);

        self.current_index += 1;
    }

    fn compile_local(&mut self, expr: &ChildExpression, name: &Token) -> Result<()> {
        Ok(())
    }

    fn compile_list_of_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<()> {
        for statement in statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statements(&mut self, statements: &Vec<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn compile_statement(&mut self, node: &ChildStatement) -> Result<()> {
        if let Some(node) = node {
            match &**node {
                Statement::Block { statements } => self.compile_statements(statements),
                Statement::Variable { name, initializer } => self.compile_variable_statement(name, initializer),
                Statement::Function { body, name, params } => self.compile_function_declaration(name, params, body),
                Statement::Expression { expression } => self.compile_expression(expression),
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                } => self.compile_conditional_statement(condition, then_branch, else_branch),
                Statement::Print { expression } => self.compile_print_statement(expression),
                Statement::Return { value } => self.compile_return_statement(value),
                Statement::While { condition, body } => self.compile_while_statement(condition, body),
            }
        } else {
            Ok(())
        }
    }

    fn compile_print_statement(&mut self, value: &ChildExpression) -> Result<()> {
        self.compile_expression(value)?;

        let function_index = self.function_names.get("log_num").ok_or_else(|| anyhow!("Unable to find print function"))?;
        self.context.instructions.push(Instruction::Call(*function_index));

        Ok(())
    }

    fn compile_return_statement(&mut self, value: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn compile_while_statement(&mut self, condition: &ChildExpression, body: &ChildStatement) -> Result<()> {
        Ok(())
    }

    fn compile_conditional_statement(&mut self, condition: &ChildExpression, then_branch: &ChildStatement, else_branch: &Option<ChildStatement>) -> Result<()> {
        Ok(())
    }

    fn compile_function_declaration(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<ChildStatement>) -> Result<()> {
        let new_context = CompileContext::init_with_params(&name.lexme, &params);
        let previous_context = std::mem::replace(&mut self.context, new_context);

        self.compile_list_of_statements(body)?;

        self.finish_function();

        self.context = previous_context;

        Ok(())
    }

    fn compile_expression(&mut self, node: &ChildExpression) -> Result<()> {
        if let Some(n) = node {
            match &**n {
                Expression::Variable { name } => self.compile_variable_expression(name, node),
                Expression::Assign { name, value } => self.compile_assign_expression(name, value, node),
                Expression::Binary { left, right, .. } => self.compile_binary(left, right),
                Expression::Call { callee, arguments } => self.compile_call_expression(callee, arguments),
                Expression::Grouping { expression } => self.compile_expression(expression),
                Expression::Literal { value } => self.compile_literal(value),
                Expression::Logical { left, right, .. } => self.compile_logical(left, right),
                Expression::Unary { right, .. } => self.compile_expression(right),
            }
        } else {
            Ok(())
        }
    }

    fn compile_literal(&mut self, literal: &TokenLiteral) -> Result<()> {
        match literal {
            TokenLiteral::Nil => todo!(),
            TokenLiteral::String(_) => todo!(),
            TokenLiteral::Number(n) => self.context.instructions.push(Instruction::F64Const(n.value())),
            TokenLiteral::Boolean(_) => todo!(),
        }
        Ok(())
    }

    fn compile_logical(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn compile_call_expression(&mut self, callee: &ChildExpression, arguments: &Vec<ChildExpression>) -> Result<()> {
        let function = match &*callee.as_ref().unwrap().as_ref() {
            Expression::Variable { name } => Ok(name),
            e => Err(anyhow!("Invalid call expression: {:?}", e)),
        }?;

        let function_index = self
            .function_names
            .get(&function.lexme)
            .ok_or_else(|| anyhow!("Unable to find function already defined"))?;
        self.context.instructions.push(Instruction::Call(*function_index));

        Ok(())
    }

    fn compile_binary(&mut self, left: &ChildExpression, right: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn compile_assign_expression(&mut self, name: &Token, value: &ChildExpression, node: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn compile_variable_expression(&mut self, name: &Token, node: &ChildExpression) -> Result<()> {
        Ok(())
    }

    fn compile_variable_statement(&mut self, name: &Token, initializer: &ChildExpression) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use claim::assert_ok;

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
        assert_eq!(Ok("42.2".to_string()), execute("42.2"));
        assert_eq!(Ok("2".to_string()), execute("2"));
    }

    #[test]
    fn clock() {
        let clock = execute("clock ()");
        assert_ok!(&clock);
        assert!(clock.unwrap().len() > 0);
    }
}
