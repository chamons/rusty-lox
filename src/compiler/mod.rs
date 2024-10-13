use std::{
    error::Error,
    fmt::{Display, Write},
};

use locals::Local;
use tracing::{error, info};

use crate::{
    bytecode::{Chunk, Instruction, Value},
    compiler::parser::Parser,
};

use tokens::token::{Token, TokenType};

pub mod parser;
pub mod tokens;

pub fn compile(source: &str) -> eyre::Result<Chunk> {
    let mut compiler = Compiler::new();
    compiler.compile(source)
}

mod locals;

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
            infix: None,
            precedence: Precedence::None,
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
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: u32,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: vec![],
            scope_depth: 0,
        }
    }

    pub fn compile(&mut self, source: &str) -> eyre::Result<Chunk> {
        self.chunk = Chunk::new();

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
            info!(chunk = %self.chunk, "Compiled chunk");
            Ok(std::mem::take(&mut self.chunk))
        }
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

    fn emit_return(&mut self, line: u32) {
        self.chunk.write(Instruction::Return, line);
    }

    fn emit_constant(&mut self, value: Value, line: u32) {
        self.chunk.write_constant(value, line);
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
                    let name_index = self.chunk.make_constant(Value::String(name.clone()));
                    (Instruction::FetchGlobal { name_index }, Instruction::SetGlobal { name_index })
                };

                if can_assign && self.match_token(parser, TokenType::Equal)? {
                    self.expression(parser)?;
                    self.chunk.write(set, parser.previous.line);
                } else {
                    self.chunk.write(get, parser.previous.line);
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

    fn unary(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        let operator_type = parser.previous.token_type.clone();

        self.parse_precedence(parser, Precedence::Unary)?;

        match operator_type {
            TokenType::Minus => self.chunk.write(Instruction::Negate, parser.previous.line),
            TokenType::Bang => self.chunk.write(Instruction::Not, parser.previous.line),
            _ => return Err(eyre::eyre!("Unexpected operator type in unary expression")),
        }

        Ok(())
    }

    fn declaration(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        if self.match_token(parser, TokenType::Var)? {
            self.variable_declaration(parser)
        } else {
            self.statement(parser)
        }
    }

    fn variable_declaration(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let variable_info = self.parse_variable(parser)?;

        self.declare_variable(&variable_info)?;

        if self.match_token(parser, TokenType::Equal)? {
            self.expression(parser)?;
        } else {
            self.chunk.write_constant(Value::Nil, parser.previous.line);
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
                self.chunk.write(Instruction::DefineGlobal { name_index: *name_index }, parser.previous.line);
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
                        name_index: self.chunk.make_constant(Value::String(identifier)),
                    })
                }
            }
            _ => Err(eyre::eyre!("Expect identifier")),
        }
    }

    fn statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        if self.match_token(parser, TokenType::Print)? {
            self.print_statement(parser)?;
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
            self.chunk.write(Instruction::Pop, parser.current.line);
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

    fn print_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::Semicolon, "Expect ';' after value.")?;
        self.chunk.write(Instruction::Print, parser.previous.line);
        Ok(())
    }

    fn expression_statement(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::Semicolon, "Expect ';' after expression.")?;
        self.chunk.write(Instruction::Pop, parser.previous.line);
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
            TokenType::Plus => self.chunk.write(Instruction::Add, parser.previous.line),
            TokenType::Minus => self.chunk.write(Instruction::Subtract, parser.previous.line),
            TokenType::Star => self.chunk.write(Instruction::Multiply, parser.previous.line),
            TokenType::Slash => self.chunk.write(Instruction::Divide, parser.previous.line),
            TokenType::BangEqual => {
                self.chunk.write(Instruction::Equal, parser.previous.line);
                self.chunk.write(Instruction::Not, parser.previous.line);
            }
            TokenType::EqualEqual => self.chunk.write(Instruction::Equal, parser.previous.line),
            TokenType::Greater => self.chunk.write(Instruction::Greater, parser.previous.line),
            TokenType::GreaterEqual => {
                self.chunk.write(Instruction::Less, parser.previous.line);
                self.chunk.write(Instruction::Not, parser.previous.line);
            }
            TokenType::Less => self.chunk.write(Instruction::Less, parser.previous.line),
            TokenType::LessEqual => {
                self.chunk.write(Instruction::Greater, parser.previous.line);
                self.chunk.write(Instruction::Not, parser.previous.line);
            }
            _ => return Err(eyre::eyre!("Unexpected operator type in binary expression")),
        }

        Ok(())
    }

    fn literal(&mut self, parser: &mut Parser, _can_assign: bool) -> eyre::Result<()> {
        match parser.previous.token_type {
            TokenType::False => self.chunk.write_constant(Value::Bool(false), parser.previous.line),
            TokenType::True => self.chunk.write_constant(Value::Bool(true), parser.previous.line),
            TokenType::Nil => self.chunk.write_constant(Value::Nil, parser.previous.line),
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
    fn compile_expected(#[case] input: String) {
        let mut compiler = Compiler::new();
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
}
