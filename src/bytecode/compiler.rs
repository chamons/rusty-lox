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

pub fn compile(script: &str) -> Result<Chunk, CompilerError> {
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
    let mut chunk = Chunk::new();
    compile_statements(&filtered_statements, &mut chunk).map_err(|e| CompilerError::ParseError(e.to_string()))?;
    chunk.write(OpCode::Return, 99);
    if cfg!(debug_assertions) {
        chunk.disassemble("code");
    }
    Ok(chunk)
}

fn compile_statements(statements: &[&Statement], chunk: &mut Chunk) -> Result<()> {
    for statement in statements {
        compile_statement(&statement, chunk)?;
    }
    Ok(())
}

fn unwrap_or_error<'a, T>(element: &'a Option<Box<T>>) -> Result<&'a T> {
    if let Some(element) = element {
        Ok(element.as_ref())
    } else {
        Err(anyhow::anyhow!("Missing expected element"))
    }
}

fn compile_statement(statement: &Statement, chunk: &mut Chunk) -> Result<()> {
    match statement {
        Statement::Expression { expression } => compile_expression(unwrap_or_error(expression)?, chunk),
        Statement::Print { expression } => todo!(),
        Statement::Variable { name, initializer } => todo!(),
        Statement::Block { statements } => todo!(),
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => todo!(),
        Statement::While { condition, body } => todo!(),
        Statement::Function { name, params, body } => todo!(),
        Statement::Return { value } => todo!(),
    }
}

fn compile_expression(expression: &Expression, chunk: &mut Chunk) -> Result<()> {
    match expression {
        Expression::Assign { name, value, line } => todo!(),
        Expression::Binary { left, operator, right, line } => {
            compile_expression(unwrap_or_error(left)?, chunk)?;
            compile_expression(unwrap_or_error(right)?, chunk)?;
            match operator.kind {
                TokenKind::Plus => {
                    chunk.write(OpCode::Add, *line);
                    Ok(())
                }
                TokenKind::Minus => {
                    chunk.write(OpCode::Subtract, *line);
                    Ok(())
                }
                TokenKind::Slash => {
                    chunk.write(OpCode::Divide, *line);
                    Ok(())
                }
                TokenKind::Star => {
                    chunk.write(OpCode::Multiply, *line);
                    Ok(())
                }
                TokenKind::Greater => {
                    chunk.write(OpCode::Greater, *line);
                    Ok(())
                }
                TokenKind::GreaterEqual => {
                    chunk.write(OpCode::Less, *line);
                    chunk.write(OpCode::Not, *line);
                    Ok(())
                }
                TokenKind::Less => {
                    chunk.write(OpCode::Less, *line);
                    Ok(())
                }
                TokenKind::LessEqual => {
                    chunk.write(OpCode::Greater, *line);
                    chunk.write(OpCode::Not, *line);
                    Ok(())
                }
                TokenKind::EqualEqual => {
                    chunk.write(OpCode::Equal, *line);
                    Ok(())
                }
                TokenKind::BangEqual => {
                    chunk.write(OpCode::Equal, *line);
                    chunk.write(OpCode::Not, *line);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Invalid binary operator")),
            }
        }
        Expression::Grouping { expression, line: _ } => compile_expression(unwrap_or_error(expression)?, chunk),
        Expression::Literal { value, line } => match value {
            TokenLiteral::Nil => {
                let index = chunk.write_value(OpValue::Nil);
                chunk.write(OpCode::Constant(index), *line);
                Ok(())
            }
            TokenLiteral::String(_) => todo!(),
            TokenLiteral::Number(v) => {
                let index = chunk.write_value(OpValue::Double(v.value()));
                chunk.write(OpCode::Constant(index), *line);
                Ok(())
            }
            TokenLiteral::Boolean(v) => {
                let index = chunk.write_value(OpValue::Boolean(*v));
                chunk.write(OpCode::Constant(index), *line);
                Ok(())
            }
        },
        Expression::Unary { operator, right, line } => {
            compile_expression(unwrap_or_error(right)?, chunk)?;
            match operator.kind {
                TokenKind::Minus => {
                    chunk.write(OpCode::Negate, *line);
                    Ok(())
                }
                TokenKind::Bang => {
                    chunk.write(OpCode::Not, *line);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Invalid binary operator")),
            }
        }
        Expression::Variable { name, line } => todo!(),
        Expression::Logical { left, operator, right, line } => todo!(),
        Expression::Call { callee, arguments, line } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_smoke_test() {
        let chunk = compile("1 + 2;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Constant(1), chunk.code[1]);
        assert_eq!(OpCode::Add, chunk.code[2]);
        assert_eq!(OpCode::Return, chunk.code[3]);
    }

    #[test]
    fn negate_smoke_test() {
        let chunk = compile("-(1);").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Negate, chunk.code[1]);
        assert_eq!(OpCode::Return, chunk.code[2]);
    }

    #[test]
    fn booleans() {
        let chunk = compile("false + true;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpCode::Constant(1), chunk.code[1]);
        assert_eq!(OpCode::Add, chunk.code[2]);
        assert_eq!(OpCode::Return, chunk.code[3]);
    }

    #[test]
    fn nil() {
        let chunk = compile("nil;").unwrap();
        assert_eq!(OpCode::Constant(0), chunk.code[0]);
        assert_eq!(OpValue::Nil, chunk.values[0]);
    }
}
