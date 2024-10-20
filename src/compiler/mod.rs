use std::{
    error::Error,
    fmt::{Display, Write},
};

use locals::Local;
use tracing::{error, info};

use crate::{
    bytecode::{Chunk, Instruction, Value},
    compiler::parser::Parser,
    vm::Function,
};

use tokens::token::{Token, TokenType};

pub mod parser;
pub mod tokens;

pub fn compile(source: &str) -> eyre::Result<Function> {
    let mut compiler = Compiler::new();
    compiler.compile(source)
}

mod locals;

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Assignment = 1, // =
    Or = 2,         // or
    And = 3,        // and
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Term = 6,       // + -
    Factor = 7,     // * /
    Unary = 8,      // ! -
    Call = 9,       // . ()
    Primary = 10,
}

impl Precedence {
    pub fn one_higher(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

type ParseFunction = fn(&mut Compiler, parser: &mut Parser, can_assign: bool) -> eyre::Result<()>;

struct ParseRule {
    prefix: Option<ParseFunction>,
    infix: Option<ParseFunction>,
    precedence: Precedence,
}

fn get_parse_rule(token_type: &TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.grouping(p, can_assign)),
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.call(p, can_assign)),
            precedence: Precedence::Call,
        },
        TokenType::Minus => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.unary(p, can_assign)),
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Term,
        },
        TokenType::Plus => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Term,
        },
        TokenType::Slash => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Factor,
        },
        TokenType::Star => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Factor,
        },
        TokenType::Number(_) => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.number(p, can_assign)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::False | TokenType::True | TokenType::Nil => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.literal(p, can_assign)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Bang => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.unary(p, can_assign)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::BangEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Equality,
        },
        TokenType::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Equality,
        },
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.binary(p, can_assign)),
            precedence: Precedence::Comparison,
        },
        TokenType::String(_) => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.string(p, can_assign)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Identifier(_) => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.variable(p, can_assign)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::And => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.and(p, can_assign)),
            precedence: Precedence::And,
        },
        TokenType::Or => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser, can_assign: bool| c.or(p, can_assign)),
            precedence: Precedence::Or,
        },
        _ => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    }
}

#[derive(Debug, Default)]
pub struct CompileErrors {
    errors: Vec<eyre::Report>,
}

impl CompileErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_any(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn push(&mut self, err: eyre::Report) {
        self.errors.push(err);
    }
}

impl Error for CompileErrors {}

impl Display for CompileErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
            error.fmt(f)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum VariableInfo {
    Global { name_index: u32 },
    Local { token: Token, depth: u32 },
}

pub struct Compiler {
    function: Function,
    function_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: u32,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            function: Function::new(),
            locals: vec![],
            scope_depth: 0,
            function_type: FunctionType::Script,
        }
    }

    pub fn new_for_function(name: String) -> Self {
        Self {
            function: Function::new_with_name(name),
            locals: vec![],
            scope_depth: 0,
            function_type: FunctionType::Function,
        }
    }

    pub fn compile(&mut self, source: &str) -> eyre::Result<Function> {
        // self.current_chunk() = Chunk::new();

        let mut parser = Parser::new(source)?;
        let mut errors = CompileErrors::new();

        while !self.match_token(&mut parser, TokenType::Eof)? {
            if let Err(err) = self.try_compile(&mut parser) {
                println!("Processing error");
                errors.push(err);
                self.synchronize(&mut parser)?;
            }
        }

        if errors.has_any() {
            info!(errors = %errors, "Error compiling chunk");
            Err(errors.into())
        } else {
            info!(chunk = %self.function, "Compiled function");
            self.end_compile(&mut parser)
        }
    }

    fn end_compile(&mut self, parser: &mut Parser) -> eyre::Result<Function> {
        self.emit_return(parser)?;
        Ok(std::mem::take(&mut self.function))
    }

    fn emit_return(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.function.chunk.write_constant(Value::Nil, parser.current.line);
        self.function.chunk.write(Instruction::Return, parser.current.line);
        Ok(())
    }

    pub fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn synchronize(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        while parser.current.token_type != TokenType::Eof {
            if parser.previous.token_type == TokenType::Semicolon {
                return Ok(());
            }
            match parser.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return Ok(());
                }
                _ => {}
            }
            parser.advance()?;
        }
        Ok(())
    }

    fn try_compile(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.declaration(parser)?;
        Ok(())
    }

    fn emit_constant(&mut self, value: Value, line: u32) {
        self.current_chunk().write_constant(value, line);
    }

    fn number(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        match &parser.previous.token_type {
            TokenType::Number(v) => {
                let number = v.parse::<f64>()?;
                self.emit_constant(Value::Double(number), parser.previous.line);
                Ok(())
            }
            _ => Err(eyre::eyre!("Unexpected token type generating number")),
        }
    }

    fn variable(&mut self, parser: &mut Parser, can_assign: bool) -> eyre::Result<()> {
        self.named_variable(parser, can_assign)
    }

    fn named_variable(&mut self, parser: &mut Parser, can_assign: bool) -> eyre::Result<()> {
        match &parser.previous.token_type {
            TokenType::Identifier(name) => {
                let local_position = self.locals.iter().rposition(|l| l.token.token_type == parser.previous.token_type);
                let (get, set) = if let Some(local_position) = local_position {
                    if !self.locals[local_position].initialized {
                        return Err(eyre::eyre!("Can't read local variable in its own initializer."));
                    }

                    (
                        Instruction::GetLocal { index: local_position as u32 },
                        Instruction::SetLocal { index: local_position as u32 },
                    )
                } else {
                    let name_index = self.current_chunk().make_constant(Value::String(name.clone()));
                    (Instruction::FetchGlobal { name_index }, Instruction::SetGlobal { name_index })
                };

                if can_assign && self.match_token(parser, TokenType::Equal)? {
                    self.expression(parser)?;
                    self.current_chunk().write(set, parser.previous.line);
                } else {
                    self.current_chunk().write(get, parser.previous.line);
                }

                Ok(())
            }
            _ => Err(eyre::eyre!("Unexpected token type generating named variable")),
        }
    }

    fn string(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        match &parser.previous.token_type {
            TokenType::String(v) => {
                self.emit_constant(Value::String(v.clone()), parser.previous.line);
                Ok(())
            }
            _ => Err(eyre::eyre!("Unexpected token type generating string")),
        }
    }

    fn grouping(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::RightParen, "Expect ')' after expression.")?;
        Ok(())
    }

    fn call(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let arg_count = self.argument_list(parser)?;
        self.current_chunk().write(Instruction::Call { arg_count }, parser.previous.line);
        Ok(())
    }

    fn argument_list(&mut self, parser: &mut Parser) -> eyre::Result<u32> {
        let mut count = 0;
        if parser.current.token_type != TokenType::RightParen {
            loop {
                self.expression(parser)?;
                count += 1;
                if !self.match_token(parser, TokenType::Comma)? {
                    break;
                }
            }
        }
        self.consume(parser, TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(count)
    }

    fn unary(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let operator_type = parser.previous.token_type.clone();

        self.parse_precedence(parser, Precedence::Unary)?;

        match operator_type {
            TokenType::Minus => self.current_chunk().write(Instruction::Negate, parser.previous.line),
            TokenType::Bang => self.current_chunk().write(Instruction::Not, parser.previous.line),
            _ => return Err(eyre::eyre!("Unexpected operator type in unary expression")),
        }

        Ok(())
    }

    fn declaration(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        if self.match_token(parser, TokenType::Fun)? {
            self.fun_declaration(parser)
        } else if self.match_token(parser, TokenType::Var)? {
            self.variable_declaration(parser)
        } else {
            self.statement(parser)
        }
    }

    fn function(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let function_name = match &parser.previous.token_type {
            TokenType::Identifier(identifier) => Ok(identifier.clone()),
            _ => Err(eyre::eyre!("Unable to find function name defined")),
        }?;

        // NOTE - Everything after this point must be compiler.Foo
        // not self.foo until we are done driving the sub-compiler
        let mut compiler = Compiler::new_for_function(function_name);

        compiler.begin_scope();
        compiler.consume(parser, TokenType::LeftParen, "Expect '(' after function name.")?;

        if parser.current.token_type != TokenType::RightParen {
            loop {
                compiler.function.arity += 1;
                if compiler.function.arity > 255 {
                    return Err(eyre::eyre!("Can't have more than 255 parameters."));
                }
                let variable_info = compiler.parse_variable(parser)?;
                compiler.declare_variable(&variable_info)?;

                compiler.define_variable(parser, &variable_info)?;
                if !compiler.match_token(parser, TokenType::Comma)? {
                    break;
                }
            }
        }

        compiler.consume(parser, TokenType::RightParen, "Expect ')' after parameters.")?;
        compiler.consume(parser, TokenType::LeftBrace, "Expect '{' before function body.")?;
        compiler.block(parser)?;

        let function = compiler.end_compile(parser)?;

        self.current_chunk()
            .write_constant(Value::Function(std::sync::Arc::new(function)), parser.previous.line);

        Ok(())
    }

    fn fun_declaration(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let variable_info = self.parse_variable(parser)?;
        self.mark_initialized();
        self.function(parser)?;
        self.define_variable(parser, &variable_info)?;
        Ok(())
    }

    fn variable_declaration(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let variable_info = self.parse_variable(parser)?;

        self.declare_variable(&variable_info)?;

        if self.match_token(parser, TokenType::Equal)? {
            self.expression(parser)?;
        } else {
            self.current_chunk().write_constant(Value::Nil, parser.previous.line);
        }

        self.define_variable(parser, &variable_info)?;

        self.consume(parser, TokenType::Semicolon, "Expect ';' after variable declaration.")?;

        Ok(())
    }

    fn declare_variable(&mut self, variable_info: &VariableInfo) -> eyre::Result<()> {
        if let VariableInfo::Local { token, depth } = variable_info {
            for local in self.locals.iter().rev() {
                if local.initialized && local.depth < *depth {
                    break;
                }
                if local.token.token_type == token.token_type {
                    return Err(eyre::eyre!("Already a variable with this name in this scope."));
                }
            }
            self.locals.push(Local {
                token: token.clone(),
                depth: *depth,
                initialized: false,
            });
        }

        Ok(())
    }

    fn define_variable(&mut self, parser: &Parser, variable_info: &VariableInfo) -> eyre::Result<()> {
        match variable_info {
            VariableInfo::Global { name_index } => {
                self.current_chunk()
                    .write(Instruction::DefineGlobal { name_index: *name_index }, parser.previous.line);
            }
            VariableInfo::Local { .. } => {
                self.mark_initialized();
            }
        }

        Ok(())
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        if let Some(last) = self.locals.last_mut() {
            last.initialized = true;
        }
    }

    fn parse_variable(&mut self, parser: &mut Parser) -> eyre::Result<VariableInfo> {
        match parser.current.token_type.clone() {
            TokenType::Identifier(identifier) => {
                parser.advance()?;
                if self.scope_depth > 0 {
                    Ok(VariableInfo::Local {
                        token: parser.previous.clone(),
                        depth: self.scope_depth,
                    })
                } else {
                    Ok(VariableInfo::Global {
                        name_index: self.current_chunk().make_constant(Value::String(identifier)),
                    })
                }
            }
            _ => Err(eyre::eyre!("Expect identifier")),
        }
    }

    fn statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        if self.match_token(parser, TokenType::Print)? {
            self.print_statement(parser)?;
        } else if self.match_token(parser, TokenType::If)? {
            self.if_statement(parser)?;
        } else if self.match_token(parser, TokenType::For)? {
            self.for_statement(parser)?;
        } else if self.match_token(parser, TokenType::Return)? {
            self.return_statement(parser)?;
        } else if self.match_token(parser, TokenType::While)? {
            self.while_statement(parser)?;
        } else if self.match_token(parser, TokenType::LeftBrace)? {
            self.begin_scope();
            self.block(parser)?;
            self.end_scope(parser);
        } else {
            self.expression_statement(parser)?;
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, parser: &Parser) {
        self.scope_depth -= 1;

        let local_to_pop = self.locals.iter().filter(|l| l.depth > self.scope_depth).count();
        for _ in 0..local_to_pop {
            self.current_chunk().write(Instruction::Pop, parser.current.line);
            self.locals.pop();
        }
    }

    fn block(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        while parser.current.token_type != TokenType::RightBrace && parser.current.token_type != TokenType::Eof {
            self.declaration(parser)?;
        }

        self.consume(parser, TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(())
    }

    fn return_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        if self.function_type == FunctionType::Script {
            return Err(eyre::eyre!("Can't return from top-level code."));
        }

        if self.match_token(parser, TokenType::Semicolon)? {
            self.emit_return(parser)?;
        } else {
            self.expression(parser)?;
            self.consume(parser, TokenType::Semicolon, "Expect ';' after return value.")?;
            self.function.chunk.write(Instruction::Return, parser.current.line);
        }
        Ok(())
    }

    fn while_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let loop_start = self.current_chunk().code.len();

        self.consume(parser, TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression(parser)?;
        self.consume(parser, TokenType::RightParen, "Expect ')' after condition.")?;

        let exit_jump = self.current_chunk().write_jump(Instruction::JumpIfFalse { offset: 0 }, parser.previous.line);
        self.current_chunk().write(Instruction::Pop, parser.previous.line);
        self.statement(parser)?;
        self.emit_loop(loop_start, &parser)?;
        self.current_chunk().patch_jump(exit_jump)?;

        self.current_chunk().write(Instruction::Pop, parser.previous.line);

        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize, parser: &Parser) -> eyre::Result<()> {
        let offset = (self.current_chunk().code.len() - loop_start + 1) as u32;
        self.current_chunk().write(Instruction::JumpBack { offset }, parser.previous.line);
        Ok(())
    }

    fn for_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.begin_scope();
        self.consume(parser, TokenType::LeftParen, "Expect '(' after 'for'.")?;

        if self.match_token(parser, TokenType::Semicolon)? {
            // No initializer
        } else if self.match_token(parser, TokenType::Var)? {
            self.variable_declaration(parser)?;
        } else {
            self.expression_statement(parser)?;
        }

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = None;
        if !self.match_token(parser, TokenType::Semicolon)? {
            self.expression(parser)?;
            self.consume(parser, TokenType::Semicolon, "Expect ';' after loop condition.")?;

            exit_jump = Some(self.current_chunk().write_jump(Instruction::JumpIfFalse { offset: 0 }, parser.previous.line));
            self.current_chunk().write(Instruction::Pop, parser.previous.line);
        }

        if !self.match_token(parser, TokenType::RightParen)? {
            let body_jump = self.current_chunk().write_jump(Instruction::Jump { offset: 0 }, parser.previous.line);
            let increment_start = self.current_chunk().code.len();
            self.expression(parser)?;
            self.current_chunk().write(Instruction::Pop, parser.previous.line);
            self.consume(parser, TokenType::RightParen, "Expect ')' after for clauses.")?;

            self.emit_loop(loop_start, parser)?;
            loop_start = increment_start;
            self.current_chunk().patch_jump(body_jump)?;
        }

        self.statement(parser)?;
        self.emit_loop(loop_start, parser)?;

        if let Some(exit_jump) = exit_jump {
            self.current_chunk().patch_jump(exit_jump)?;
            self.current_chunk().write(Instruction::Pop, parser.previous.line);
        }

        self.end_scope(parser);
        Ok(())
    }

    fn if_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.consume(parser, TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression(parser)?;
        self.consume(parser, TokenType::RightParen, "Expect ')' after condition.")?;

        let then_jump = self.current_chunk().write_jump(Instruction::JumpIfFalse { offset: 0 }, parser.previous.line);
        self.current_chunk().write(Instruction::Pop, parser.previous.line);
        self.statement(parser)?;

        let else_jump = self.current_chunk().write_jump(Instruction::Jump { offset: 0 }, parser.previous.line);

        self.current_chunk().patch_jump(then_jump)?;
        self.current_chunk().write(Instruction::Pop, parser.previous.line);

        if self.match_token(parser, TokenType::Else)? {
            self.statement(parser)?;
        }

        self.current_chunk().patch_jump(else_jump)?;

        Ok(())
    }

    fn and(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let end_jump = self.current_chunk().write_jump(Instruction::JumpIfFalse { offset: 0 }, parser.previous.line);
        self.current_chunk().write(Instruction::Pop, parser.previous.line);
        self.parse_precedence(parser, Precedence::And)?;
        self.current_chunk().patch_jump(end_jump)?;
        Ok(())
    }

    fn or(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let else_jump = self.current_chunk().write_jump(Instruction::JumpIfFalse { offset: 0 }, parser.previous.line);
        let end_jump = self.current_chunk().write_jump(Instruction::Jump { offset: 0 }, parser.previous.line);

        self.current_chunk().patch_jump(else_jump)?;

        self.current_chunk().write(Instruction::Pop, parser.previous.line);
        self.parse_precedence(parser, Precedence::Or)?;
        self.current_chunk().patch_jump(end_jump)?;

        Ok(())
    }

    fn print_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::Semicolon, "Expect ';' after value.")?;
        self.current_chunk().write(Instruction::Print, parser.previous.line);
        Ok(())
    }

    fn expression_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::Semicolon, "Expect ';' after expression.")?;
        self.current_chunk().write(Instruction::Pop, parser.previous.line);
        Ok(())
    }

    fn expression(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.parse_precedence(parser, Precedence::Assignment)
    }

    fn binary(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let operator_type = parser.previous.token_type.clone();

        let rule = get_parse_rule(&operator_type);

        self.parse_precedence(parser, rule.precedence.one_higher())?;

        match operator_type {
            TokenType::Plus => self.current_chunk().write(Instruction::Add, parser.previous.line),
            TokenType::Minus => self.current_chunk().write(Instruction::Subtract, parser.previous.line),
            TokenType::Star => self.current_chunk().write(Instruction::Multiply, parser.previous.line),
            TokenType::Slash => self.current_chunk().write(Instruction::Divide, parser.previous.line),
            TokenType::BangEqual => {
                self.current_chunk().write(Instruction::Equal, parser.previous.line);
                self.current_chunk().write(Instruction::Not, parser.previous.line);
            }
            TokenType::EqualEqual => self.current_chunk().write(Instruction::Equal, parser.previous.line),
            TokenType::Greater => self.current_chunk().write(Instruction::Greater, parser.previous.line),
            TokenType::GreaterEqual => {
                self.current_chunk().write(Instruction::Less, parser.previous.line);
                self.current_chunk().write(Instruction::Not, parser.previous.line);
            }
            TokenType::Less => self.current_chunk().write(Instruction::Less, parser.previous.line),
            TokenType::LessEqual => {
                self.current_chunk().write(Instruction::Greater, parser.previous.line);
                self.current_chunk().write(Instruction::Not, parser.previous.line);
            }
            _ => return Err(eyre::eyre!("Unexpected operator type in binary expression")),
        }

        Ok(())
    }

    fn literal(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        match parser.previous.token_type {
            TokenType::False => self.current_chunk().write_constant(Value::Bool(false), parser.previous.line),
            TokenType::True => self.current_chunk().write_constant(Value::Bool(true), parser.previous.line),
            TokenType::Nil => self.current_chunk().write_constant(Value::Nil, parser.previous.line),
            _ => return Err(eyre::eyre!("Unexpected type in literal expression")),
        }
        Ok(())
    }

    fn parse_precedence(&mut self, parser: &mut Parser, precedence: Precedence) -> eyre::Result<()> {
        parser.advance()?;

        info!(previous = ?parser.previous.token_type, current = ?parser.current.token_type, "parse_precedence");

        let rule = get_parse_rule(&parser.previous.token_type);
        let can_assign = precedence <= Precedence::Assignment;

        if let Some(prefix) = &rule.prefix {
            prefix(self, parser, can_assign)?;
        } else {
            return Err(eyre::eyre!("Expect expression"));
        }

        while precedence <= get_parse_rule(&parser.current.token_type).precedence {
            parser.advance()?;
            let rule = get_parse_rule(&parser.previous.token_type);
            info!(precedence = ?rule.precedence, "parse_precedence inner");

            if let Some(infix) = &rule.infix {
                infix(self, parser, can_assign)?;
            } else {
                return Err(eyre::eyre!("Expect expression"));
            }
        }

        if can_assign && self.match_token(parser, TokenType::Equal)? {
            return Err(eyre::eyre!("Invalid assignment target."));
        }

        Ok(())
    }

    fn consume(&mut self, parser: &mut Parser, token: TokenType, message: &str) -> eyre::Result<()> {
        if parser.current.token_type == token {
            parser.advance()?;
            return Ok(());
        }

        error!(expected = ?token, current = ?parser.current.token_type, "Unable to consume expected type");
        Err(eyre::eyre!(message.to_string()))
    }

    fn match_token(&mut self, parser: &mut Parser, token: TokenType) -> eyre::Result<bool> {
        if parser.current.token_type == token {
            parser.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::bytecode::{Instruction, Value};

    use super::Compiler;

    #[rstest]
    #[case("1 + 2;")]
    #[case("(1 + 2);")]
    #[case("(-1 + 2) * 3 - -4;")]
    #[case("true;")]
    #[case("false;")]
    #[case("nil;")]
    #[case("!false;")]
    #[case("var x = 42;")]
    #[case("{ var x = 42; }")]
    #[case(
        "{
      var a = \"outer\";
      {
        var a = \"inner\";
      }
    }"
    )]
    #[case("fun f () {}")]
    #[case("fun f (a) {}")]
    #[case("fun f (a, b) {}")]
    #[case("fun f(b) {} f(1);")]
    fn compile_expected(#[case] input: String) {
        let mut compiler = Compiler::new();
        println!("{input}");
        compiler.compile(&input).unwrap();
    }

    #[rstest]
    #[case("a * b = c + d;")]
    #[case(
        "{
  var a = \"first\";
  var a = \"second\";
}"
    )]
    #[case("return 0;")]
    fn compile_fails(#[case] input: String) {
        let mut compiler = Compiler::new();
        assert!(compiler.compile(&input).is_err());
    }

    #[test]
    fn locals_scoping() {
        let mut compiler = Compiler::new();
        compiler
            .compile(
                "{
  var a = \"outer\";
  {
    var a = \"inner\";
  }
}",
            )
            .unwrap();
        assert_eq!(0, compiler.locals.len());
    }

    #[test]
    fn locals_scoping_redeclare_referencing_same() {
        let mut compiler = Compiler::new();
        assert!(compiler
            .compile(
                "{
  var a = \"outer\";
  {
    var a = a;
  }
}",
            )
            .is_err());
    }

    #[test]
    fn local_function_arg() {
        let mut compiler = Compiler::new();
        let function = compiler
            .compile(
                "fun second(b) {
  var c = 3;
  var d = 4;
  print b;
}
",
            )
            .unwrap();
        let second = match function.chunk.constant(1) {
            Value::Function(second) => second,
            _ => panic!(),
        };
        assert!(matches!(second.chunk.code[2], Instruction::GetLocal { index: 0 }));
    }
}
