use std::collections::HashMap;

use crate::interpreter::InterpreterLiteral;

pub struct Environment {
    values: HashMap<String, InterpreterLiteral>,
}

impl Environment {
    pub fn init() -> Self {
        Environment { values: HashMap::new() }
    }

    pub fn define(&mut self, name: &String, value: InterpreterLiteral) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &String) -> Option<&InterpreterLiteral> {
        self.values.get(name)
    }

    pub fn assign(&mut self, name: &str, value: InterpreterLiteral) -> Result<(), &'static str> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else {
            Err("Undefined variable usage.")
        }
    }
}
