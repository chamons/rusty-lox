use float_cmp::approx_eq;

use crate::expressions::*;
use crate::parser::*;
use crate::tokens::*;

#[derive(Debug, Clone)]
pub enum InterpreterLiteral {
    Nil,
    String(String),
    Number(f64),
    Boolean(bool),
}

impl PartialEq for InterpreterLiteral {
    fn eq(&self, other: &Self) -> bool {
        match self {
            InterpreterLiteral::Nil => matches!(other, InterpreterLiteral::Nil),
            InterpreterLiteral::String(v) => match other {
                InterpreterLiteral::String(v2) => *v == *v2,
                _ => false,
            },
            InterpreterLiteral::Number(v) => match other {
                InterpreterLiteral::Number(v2) => approx_eq!(f64, *v, *v2),
                _ => false,
            },
            InterpreterLiteral::Boolean(v) => match other {
                InterpreterLiteral::Boolean(v2) => *v == *v2,
                _ => false,
            },
        }
    }
}
impl Eq for InterpreterLiteral {}

fn expect_literal(value: &InterpreterLiteral) -> Result<f64, &'static str> {
    match value {
        InterpreterLiteral::Number(v) => Ok(*v),
        _ => Err("Operand must be a number"),
    }
}

fn expect_string<'a>(value: &'a InterpreterLiteral) -> Result<&'a str, &'static str> {
    match value {
        InterpreterLiteral::String(v) => Ok(v),
        _ => Err("Operand must be a string"),
    }
}

pub fn is_truthy(value: &InterpreterLiteral) -> bool {
    match value {
        InterpreterLiteral::Nil => false,
        InterpreterLiteral::Boolean(v) => *v,
        _ => true,
    }
}

struct Interpreter<'a> {
    root: &'a ChildExpression,
}

impl<'a> Interpreter<'a> {
    pub fn init(root: &'a ChildExpression) -> Self {
        Interpreter { root }
    }

    pub fn execute(&mut self) -> Result<InterpreterLiteral, &'static str> {
        Interpreter::execute_node(&self.root)
    }

    pub fn execute_binary(left: &ChildExpression, operator: &Token, right: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let left = Interpreter::execute_node(left)?;
        let right = Interpreter::execute_node(right)?;
        match operator.kind {
            TokenKind::Plus => {
                if matches!(left, InterpreterLiteral::Number(_)) && matches!(right, InterpreterLiteral::Number(_)) {
                    Ok(InterpreterLiteral::Number(expect_literal(&left)? + expect_literal(&right)?))
                } else if matches!(left, InterpreterLiteral::String(_)) && matches!(right, InterpreterLiteral::String(_)) {
                    Ok(InterpreterLiteral::String(format!("{}{}", expect_string(&left)?, expect_string(&right)?)))
                } else {
                    Err("Invalid addition operator arguments")
                }
            }
            TokenKind::Minus => Ok(InterpreterLiteral::Number(expect_literal(&left)? - expect_literal(&right)?)),
            TokenKind::Slash => Ok(InterpreterLiteral::Number(expect_literal(&left)? / expect_literal(&right)?)),
            TokenKind::Star => Ok(InterpreterLiteral::Number(expect_literal(&left)? * expect_literal(&right)?)),
            TokenKind::Greater => Ok(InterpreterLiteral::Boolean(expect_literal(&left)? > expect_literal(&right)?)),
            TokenKind::GreaterEqual => Ok(InterpreterLiteral::Boolean(expect_literal(&left)? >= expect_literal(&right)?)),
            TokenKind::Less => Ok(InterpreterLiteral::Boolean(expect_literal(&left)? < expect_literal(&right)?)),
            TokenKind::LessEqual => Ok(InterpreterLiteral::Boolean(expect_literal(&left)? <= expect_literal(&right)?)),
            TokenKind::EqualEqual => Ok(InterpreterLiteral::Boolean(left == right)),
            TokenKind::BangEqual => Ok(InterpreterLiteral::Boolean(left != right)),
            _ => Err("Invalid binary operator"),
        }
    }

    pub fn execute_grouping(expression: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        Interpreter::execute_node(expression)
    }

    pub fn execute_literal(value: &TokenLiteral) -> Result<InterpreterLiteral, &'static str> {
        match value {
            TokenLiteral::Nil => Ok(InterpreterLiteral::Nil),
            TokenLiteral::String(v) => Ok(InterpreterLiteral::String(v.to_string())),
            TokenLiteral::Number(v) => Ok(InterpreterLiteral::Number(*v)),
            TokenLiteral::Boolean(v) => Ok(InterpreterLiteral::Boolean(*v)),
        }
    }

    pub fn execute_unary(operator: &Token, right: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let right = Interpreter::execute_node(right)?;
        match operator.kind {
            TokenKind::Minus => Ok(InterpreterLiteral::Number(expect_literal(&right)? * -1.0)),
            TokenKind::Bang => Ok(InterpreterLiteral::Boolean(!is_truthy(&right))),
            _ => Err("Invalid unary operator"),
        }
    }

    pub fn execute_node(node: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        if let Some(node) = node {
            match &**node {
                Expression::Binary { left, operator, right } => Interpreter::execute_binary(&left, &operator, &right),
                Expression::Grouping { expression } => Interpreter::execute_grouping(&expression),
                Expression::Literal { value } => Interpreter::execute_literal(&value),
                Expression::Unary { operator, right } => Interpreter::execute_unary(&operator, &right),
            }
        } else {
            Ok(InterpreterLiteral::Nil)
        }
    }
}

pub fn run(script: &str) {
    let mut scanner = Scanner::init(script);
    let (tokens, errors) = scanner.scan_tokens();
    if errors.len() > 0 {
        return;
    }

    let mut parser = Parser::init(tokens);
    match parser.parse() {
        Ok(expression) => {
            // println!("Tree: {}", print_tree(&expression));
            let mut interpreter = Interpreter::init(&expression);
            match interpreter.execute() {
                Ok(result) => match result {
                    InterpreterLiteral::Nil => println!("nil"),
                    InterpreterLiteral::String(v) => println!("{}", v),
                    InterpreterLiteral::Number(v) => println!("{}", v),
                    InterpreterLiteral::Boolean(v) => println!("{}", v),
                },
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn literal_equality() {
        assert_eq!(InterpreterLiteral::Number(42.0), InterpreterLiteral::Number(42.0));
        assert_ne!(InterpreterLiteral::Number(42.0), InterpreterLiteral::Number(42.1));
        assert_eq!(Ok(InterpreterLiteral::String("asdf".to_string())), execute("\"asdf\""));
        assert_ne!(Ok(InterpreterLiteral::String("asdf".to_string())), execute("\"asd\""));
        assert_eq!(Ok(InterpreterLiteral::Boolean(true)), execute("true"));
        assert_eq!(Ok(InterpreterLiteral::Boolean(false)), execute("false"));
        assert_ne!(Ok(InterpreterLiteral::Boolean(false)), execute("true"));
        assert_eq!(Ok(InterpreterLiteral::Nil), execute("nil"));
    }

    fn execute(script: &str) -> Result<InterpreterLiteral, &'static str> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        let results = parser.parse().unwrap();
        Interpreter::init(&results).execute()
    }

    #[test]
    fn single_values() {
        assert_eq!(InterpreterLiteral::Number(42.0), execute("42").ok().unwrap());
        assert_eq!(InterpreterLiteral::String("asdf".to_string()), execute("\"asdf\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("true").ok().unwrap());
        assert_eq!(InterpreterLiteral::Nil, execute("nil").ok().unwrap());
    }

    #[test]
    fn negative() {
        assert_eq!(InterpreterLiteral::Number(-42.0), execute("-42").ok().unwrap());
        assert!(execute("-\"asdf\"").is_err());
        assert!(execute("-nil").is_err());
        assert!(execute("-false").is_err());
    }

    #[test]
    fn bang() {
        assert_eq!(InterpreterLiteral::Boolean(true), execute("!false").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("!true").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("!nil").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("!42").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("!\"a\"").ok().unwrap());
    }

    #[test]
    fn binary() {
        assert_eq!(InterpreterLiteral::Number(5.0), execute("3 + 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::String("32".to_string()), execute("\"3\" + \"2\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Number(1.0), execute("3 - 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Number(2.0), execute("4 / 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Number(8.0), execute("4 * 2").ok().unwrap());
        assert!(execute("4 * false").is_err());
        assert!(execute("4 / nil").is_err());
        assert!(execute("4 - nil").is_err());
        assert!(execute("4 + nil").is_err());
        assert!(execute("\"4\" + nil").is_err());
    }

    #[test]
    fn comparison() {
        assert_eq!(InterpreterLiteral::Boolean(true), execute("3 > 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("1 >= 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("3 <= 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("3 < 4").ok().unwrap());
        assert!(execute("3 < false").is_err());
        assert!(execute("3 >= nil").is_err());

        assert_eq!(InterpreterLiteral::Boolean(false), execute("3 == 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("3 != 2").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("false == true").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("false != true").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("\"a\" == \"b\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("\"a\" != \"b\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("3 == false").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("3 == nil").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(false), execute("3 == \"3\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("nil == nil").ok().unwrap());
    }
}
