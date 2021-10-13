use std::fmt;

use float_cmp::approx_eq;

use crate::environment::Environment;
use crate::expressions::*;
use crate::parser::*;
use crate::statements::*;
use crate::tokens::*;

#[derive(Debug, Clone)]
pub enum InterpreterLiteral {
    Nil,
    String(String),
    Number(f64),
    Boolean(bool),
}

impl fmt::Display for InterpreterLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpreterLiteral::Nil => write!(f, "nil"),
            InterpreterLiteral::String(v) => write!(f, "{}", v),
            InterpreterLiteral::Number(v) => write!(f, "{}", v),
            InterpreterLiteral::Boolean(v) => write!(f, "{}", v),
        }
    }
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

pub struct Interpreter<T>
where
    T: FnMut(&InterpreterLiteral),
{
    environment: Environment,
    print: T,
}

impl<T> Interpreter<T>
where
    T: FnMut(&InterpreterLiteral),
{
    pub fn init(print: T) -> Self {
        Interpreter {
            print,
            environment: Environment::init(),
        }
    }

    pub fn execute(&mut self, statements: &Vec<ChildStatement>) -> Result<(), &'static str> {
        for statement in statements {
            self.execute_statement(&statement)?;
        }
        Ok(())
    }

    pub fn execute_binary(&mut self, left: &ChildExpression, operator: &Token, right: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let left = self.execute_expression(left)?;
        let right = self.execute_expression(right)?;
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

    pub fn execute_variable_statement(&mut self, name: &Token, initializer: &Option<ChildExpression>) -> Result<InterpreterLiteral, &'static str> {
        let value = if let Some(initializer) = initializer {
            self.execute_expression(initializer)?
        } else {
            InterpreterLiteral::Nil
        };

        self.environment.define(&name.lexme, value);

        Ok(InterpreterLiteral::Nil)
    }

    pub fn execute_print_statement(&mut self, expression: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let value = self.execute_expression(expression)?;
        (self.print)(&value);
        Ok(InterpreterLiteral::Nil)
    }

    pub fn execute_expression_statement(&mut self, expression: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        self.execute_expression(expression)?;
        Ok(InterpreterLiteral::Nil)
    }

    pub fn execute_grouping(&mut self, expression: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        self.execute_expression(expression)
    }

    pub fn execute_literal(&mut self, value: &TokenLiteral) -> Result<InterpreterLiteral, &'static str> {
        match value {
            TokenLiteral::Nil => Ok(InterpreterLiteral::Nil),
            TokenLiteral::String(v) => Ok(InterpreterLiteral::String(v.to_string())),
            TokenLiteral::Number(v) => Ok(InterpreterLiteral::Number(*v)),
            TokenLiteral::Boolean(v) => Ok(InterpreterLiteral::Boolean(*v)),
        }
    }

    pub fn execute_unary(&mut self, operator: &Token, right: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let right = self.execute_expression(right)?;
        match operator.kind {
            TokenKind::Minus => Ok(InterpreterLiteral::Number(expect_literal(&right)? * -1.0)),
            TokenKind::Bang => Ok(InterpreterLiteral::Boolean(!is_truthy(&right))),
            _ => Err("Invalid unary operator"),
        }
    }

    pub fn execute_expression(&mut self, node: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        if let Some(node) = node {
            match &**node {
                Expression::Binary { left, operator, right } => self.execute_binary(&left, &operator, &right),
                Expression::Grouping { expression } => self.execute_grouping(&expression),
                Expression::Literal { value } => self.execute_literal(&value),
                Expression::Unary { operator, right } => self.execute_unary(&operator, &right),
                Expression::Variable { name } => match self.environment.get(&name.lexme) {
                    Some(v) => Ok(v.clone()),
                    None => Err(""),
                },
            }
        } else {
            Ok(InterpreterLiteral::Nil)
        }
    }

    pub fn execute_statement(&mut self, node: &ChildStatement) -> Result<InterpreterLiteral, &'static str> {
        if let Some(node) = node {
            match &**node {
                Statements::Expression { expression } => self.execute_expression_statement(&expression),
                Statements::Print { expression } => self.execute_print_statement(&expression),
                Statements::Variable { name, initializer } => self.execute_variable_statement(&name, &initializer),
            }
        } else {
            Ok(InterpreterLiteral::Nil)
        }
    }
}

pub fn run_script(script: &str) {
    let mut scanner = Scanner::init(script);
    let (tokens, errors) = scanner.scan_tokens();
    if errors.len() > 0 {
        for e in errors {
            println!("{}", e);
        }
        return;
    }
    let mut parser = Parser::init(tokens);
    match parser.parse() {
        Ok(statements) => {
            let mut interpreter = Interpreter::init(|p| println!("{}", p));
            match interpreter.execute(&statements) {
                Err(err) => {
                    println!("Error: {}", err);
                }
                _ => {}
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
        let mut scanner = Scanner::init(&format!("print {};", script));
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        let parsed = parser.parse().unwrap();
        let mut value = None;
        let mut interpreter = Interpreter::init(|p| {
            value = Some(p.clone());
        });
        interpreter.execute(&parsed)?;
        Ok(value.unwrap_or(InterpreterLiteral::Nil))
    }

    fn execute_no_redirect(script: &str) -> Result<(), &'static str> {
        let mut scanner = Scanner::init(script);
        let (tokens, errors) = scanner.scan_tokens();
        assert_eq!(0, errors.len());

        let mut parser = Parser::init(tokens);
        let parsed = parser.parse().unwrap();
        let mut interpreter = Interpreter::init(|_| {});
        interpreter.execute(&parsed)
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

    #[test]
    fn print() {
        execute_no_redirect("print \"three\";").ok();
        execute_no_redirect("print 3;").ok();
        execute_no_redirect("print nil;").ok();
    }
}
