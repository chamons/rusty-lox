use tracing::{error, info};

use crate::{
    bytecode::{Chunk, Instruction, Value},
    compiler::parser::Parser,
};

use super::tokens::token::TokenType;

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

type ParseFunction = fn(&mut Compiler, parser: &mut Parser) -> eyre::Result<()>;

struct ParseRule {
    prefix: Option<ParseFunction>,
    infix: Option<ParseFunction>,
    precedence: Precedence,
}

fn get_parse_rule(token_type: &TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.grouping(p)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Minus => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.unary(p)),
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Term,
        },
        TokenType::Plus => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Term,
        },
        TokenType::Slash => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Factor,
        },
        TokenType::Star => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Factor,
        },
        TokenType::Number(_) => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.number(p)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::False | TokenType::True | TokenType::Nil => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.literal(p)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Bang => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.unary(p)),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::BangEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Equality,
        },
        TokenType::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Equality,
        },
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => ParseRule {
            prefix: None,
            infix: Some(|c: &mut Compiler, p: &mut Parser| c.binary(p)),
            precedence: Precedence::Comparison,
        },
        TokenType::String(_) => ParseRule {
            prefix: Some(|c: &mut Compiler, p: &mut Parser| c.string(p)),
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

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Self { chunk: Chunk::new() }
    }

    pub fn compile(&mut self, source: &str) -> eyre::Result<Chunk> {
        self.chunk = Chunk::new();

        let mut parser = Parser::new(source)?;

        while !self.match_token(&mut parser, TokenType::Eof)? {
            self.expression(&mut parser)?;
        }

        info!(chunk = %self.chunk, "Compiled chunk");

        Ok(std::mem::take(&mut self.chunk))
    }

    fn emit_return(&mut self, line: u32) {
        self.chunk.write(Instruction::Return, line);
    }

    fn emit_constant(&mut self, value: Value, line: u32) {
        self.chunk.write_constant(value, line);
    }

    fn number(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        match &parser.previous.token_type {
            TokenType::Number(v) => {
                let number = v.parse::<f64>()?;
                self.emit_constant(Value::Double(number), parser.previous.line);
                Ok(())
            }
            _ => Err(eyre::eyre!("Unexpected token type generating number")),
        }
    }

    fn string(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        match &parser.previous.token_type {
            TokenType::String(v) => {
                self.emit_constant(Value::String(v.clone()), parser.previous.line);
                Ok(())
            }
            _ => Err(eyre::eyre!("Unexpected token type generating string")),
        }
    }

    fn grouping(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.expression(parser)?;
        self.consume(parser, TokenType::RightParen, "Expect ')' after expression.")?;
        Ok(())
    }

    fn unary(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        let operator_type = parser.previous.token_type.clone();

        self.parse_precedence(parser, Precedence::Unary)?;

        match operator_type {
            TokenType::Minus => self.chunk.write(Instruction::Negate, parser.previous.line),
            TokenType::Bang => self.chunk.write(Instruction::Not, parser.previous.line),
            _ => return Err(eyre::eyre!("Unexpected operator type in unary expression")),
        }

        Ok(())
    }

    fn expression(&mut self, parser: &mut Parser) -> eyre::Result<()> {
        self.parse_precedence(parser, Precedence::Assignment)
    }

    fn binary(&mut self, parser: &mut Parser) -> eyre::Result<()> {
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

    fn literal(&mut self, parser: &mut Parser) -> eyre::Result<()> {
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

        if let Some(prefix) = &rule.prefix {
            prefix(self, parser)?;
        } else {
            return Err(eyre::eyre!("Expect expression"));
        }

        while precedence <= get_parse_rule(&parser.current.token_type).precedence {
            parser.advance()?;
            let rule = get_parse_rule(&parser.previous.token_type);
            info!(precedence = ?rule.precedence, "parse_precedence inner");

            if let Some(infix) = &rule.infix {
                infix(self, parser)?;
            } else {
                return Err(eyre::eyre!("Expect expression"));
            }
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
    #[case("1 + 2")]
    #[case("(1 + 2)")]
    #[case("(-1 + 2) * 3 - -4")]
    #[case("true")]
    #[case("false")]
    #[case("nil")]
    #[case("!false")]
    fn compile_expected(#[case] input: String) {
        let mut compiler = Compiler::new();
        compiler.compile(&input).unwrap();
    }

    #[rstest]
    #[case("-false")]
    fn compile_fails(#[case] input: String) {
        let mut compiler = Compiler::new();
        compiler.compile(&input).unwrap();
    }
}
