use anyhow::Result;
use thiserror::Error;

use crate::parser::*;

use super::*;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Compile Scanner Error: {0}")]
    ScannerError(String),
    #[error("Compile Parse Error: {0}")]
    ParseError(String),
}

fn unwrap_or_error<T>(element: &Option<Box<T>>) -> Result<&T> {
    if let Some(element) = element {
        Ok(element.as_ref())
    } else {
        Err(anyhow::anyhow!("Missing expected element"))
    }
}

struct Compiler {
    pub chunk: Chunk,
    pub strings: Interner,
}

pub fn compile(script: &str) -> Result<(Chunk, Interner), CompilerError> {
    let mut compiler = Compiler::new();
    compiler.compile(script)?;
    Ok((compiler.chunk, compiler.strings))
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            strings: Interner::new(),
        }
    }

    pub fn compile(&mut self, script: &str) -> Result<(), CompilerError> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        if errors.len() > 0 {
            return Err(CompilerError::ScannerError(format!("{:?}", errors)));
        }
        let mut parser = Parser::init(tokens);
        let statements = match parser.parse() {
            Ok(statements) => statements,
            Err(e) => {
                return Err(CompilerError::ParseError(e.to_string()));
            }
        };
        // FIXME - The parser's API returns optionals that are ugly to filter out
        let filtered_statements: Vec<&Statement> = statements.iter().filter(|s| s.is_some()).map(|s| s.as_ref().unwrap().as_ref()).collect();

        self.compile_statements(&filtered_statements)
            .map_err(|e| CompilerError::ParseError(e.to_string()))?;
        self.chunk.write(OpCode::Return, 99);
        if cfg!(debug_assertions) {
            self.chunk.disassemble("code");
        }
        Ok(())
    }

    fn compile_statements(&mut self, statements: &[&Statement]) -> Result<()> {
        for statement in statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression { expression } => self.compile_expression(unwrap_or_error(expression)?),
            Statement::Print { expression: _ } => todo!(),
            Statement::Variable { name: _, initializer: _ } => todo!(),
            Statement::Block { statements: _ } => todo!(),
            Statement::If {
                condition: _,
                then_branch: _,
                else_branch: _,
            } => todo!(),
            Statement::While { condition: _, body: _ } => todo!(),
            Statement::Function { name: _, params: _, body: _ } => todo!(),
            Statement::Return { value: _ } => todo!(),
        }
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::Assign { name: _, value: _, line: _ } => todo!(),
            Expression::Binary { left, operator, right, line } => {
                self.compile_expression(unwrap_or_error(left)?)?;
                self.compile_expression(unwrap_or_error(right)?)?;
                match operator.kind {
                    TokenKind::Plus => {
                        self.chunk.write(OpCode::Add, *line);
                        Ok(())
                    }
                    TokenKind::Minus => {
                        self.chunk.write(OpCode::Subtract, *line);
                        Ok(())
                    }
                    TokenKind::Slash => {
                        self.chunk.write(OpCode::Divide, *line);
                        Ok(())
                    }
                    TokenKind::Star => {
                        self.chunk.write(OpCode::Multiply, *line);
                        Ok(())
                    }
                    TokenKind::Greater => {
                        self.chunk.write(OpCode::Greater, *line);
                        Ok(())
                    }
                    TokenKind::GreaterEqual => {
                        self.chunk.write(OpCode::Less, *line);
                        self.chunk.write(OpCode::Not, *line);
                        Ok(())
                    }
                    TokenKind::Less => {
                        self.chunk.write(OpCode::Less, *line);
                        Ok(())
                    }
                    TokenKind::LessEqual => {
                        self.chunk.write(OpCode::Greater, *line);
                        self.chunk.write(OpCode::Not, *line);
                        Ok(())
                    }
                    TokenKind::EqualEqual => {
                        self.chunk.write(OpCode::Equal, *line);
                        Ok(())
                    }
                    TokenKind::BangEqual => {
                        self.chunk.write(OpCode::Equal, *line);
                        self.chunk.write(OpCode::Not, *line);
                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!("Invalid binary operator")),
                }
            }
            Expression::Grouping { expression, line: _ } => self.compile_expression(unwrap_or_error(expression)?),
            Expression::Literal { value, line } => match value {
                TokenLiteral::Nil => {
                    let index = self.chunk.write_value(OpValue::Nil);
                    self.chunk.write(OpCode::Constant(index), *line);
                    Ok(())
                }
                TokenLiteral::String(v) => {
                    let index = self.chunk.write_value(OpValue::Object(ObjectType::String(self.strings.intern(v))));
                    self.chunk.write(OpCode::Constant(index), *line);
                    Ok(())
                }
                TokenLiteral::Number(v) => {
                    let index = self.chunk.write_value(OpValue::Double(v.value()));
                    self.chunk.write(OpCode::Constant(index), *line);
                    Ok(())
                }
                TokenLiteral::Boolean(v) => {
                    let index = self.chunk.write_value(OpValue::Boolean(*v));
                    self.chunk.write(OpCode::Constant(index), *line);
                    Ok(())
                }
            },
            Expression::Unary { operator, right, line } => {
                self.compile_expression(unwrap_or_error(right)?)?;
                match operator.kind {
                    TokenKind::Minus => {
                        self.chunk.write(OpCode::Negate, *line);
                        Ok(())
                    }
                    TokenKind::Bang => {
                        self.chunk.write(OpCode::Not, *line);
                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!("Invalid binary operator")),
                }
            }
            Expression::Variable { name: _, line: _ } => todo!(),
            Expression::Logical {
                left: _,
                operator: _,
                right: _,
                line: _,
            } => todo!(),
            Expression::Call {
                callee: _,
                arguments: _,
                line: _,
            } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_smoke_test() {
        let (chunk, _) = compile("1 + 2;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Constant(1), chunk.code[1]);
        assert_eq!(OpCode::Add, chunk.code[2]);
        assert_eq!(OpCode::Return, chunk.code[3]);
    }

    #[test]
    fn negate_smoke_test() {
        let (chunk, _) = compile("-(1);").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Negate, chunk.code[1]);
        assert_eq!(OpCode::Return, chunk.code[2]);
    }

    #[test]
    fn booleans() {
        let (chunk, _) = compile("false + true;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Constant(1), chunk.code[1]);
        assert_eq!(OpCode::Add, chunk.code[2]);
        assert_eq!(OpCode::Return, chunk.code[3]);
    }

    #[test]
    fn nil() {
        let (chunk, _) = compile("nil;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpValue::Nil, chunk.values[0]);
    }

    #[test]
    fn string() {
        let (chunk, _) = compile("\"asdf\";").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert!(matches!(&chunk.values[0], OpValue::Object(_)));
    }
}
