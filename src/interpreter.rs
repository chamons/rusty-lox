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

    fn expect_literal(value: &InterpreterLiteral) -> Result<f64, &'static str> {
        match value {
            InterpreterLiteral::Number(v) => Ok(*v),
            _ => Err("Operand must be a number"),
        }
    }

    pub fn execute_unary(operator: &Token, right: &ChildExpression) -> Result<InterpreterLiteral, &'static str> {
        let right = Interpreter::execute_node(right)?;
        match operator.kind {
            TokenKind::Minus => {
                let value = Interpreter::expect_literal(&right)?;
                Ok(InterpreterLiteral::Number(value * -1.0))
            }
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
            println!("Tree: {}", print_tree(&expression));
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
    use float_cmp::approx_eq;

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
    pub fn single_values() {
        assert_eq!(InterpreterLiteral::Number(42.0), execute("42").ok().unwrap());
        assert_eq!(InterpreterLiteral::String("asdf".to_string()), execute("\"asdf\"").ok().unwrap());
        assert_eq!(InterpreterLiteral::Boolean(true), execute("true").ok().unwrap());
        assert_eq!(InterpreterLiteral::Nil, execute("nil").ok().unwrap());
    }
}
