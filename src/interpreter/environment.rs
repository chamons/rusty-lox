use crate::interpreter::InterpreterLiteral;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Environment {
    values: HashMap<String, InterpreterLiteral>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn init() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn init_with_parent(parent: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }
    }

    pub fn define(&mut self, name: &str, value: InterpreterLiteral) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<InterpreterLiteral> {
        match self.values.get(name) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(parent) = &self.parent {
                    parent.borrow().get(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_at(me: &Rc<RefCell<Environment>>, distance: usize, name: &str) -> Option<InterpreterLiteral> {
        Environment::ancestor(me, distance).borrow().get(name)
    }

    fn ancestor(me: &Rc<RefCell<Environment>>, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment: Rc<RefCell<Environment>> = Rc::clone(me);
        for _ in 0..distance {
            let parent = Rc::clone(
                environment
                    .borrow()
                    .parent
                    .as_ref()
                    .expect("Walked up an invalid number of levels in environment"),
            );
            environment = parent;
        }
        environment
    }

    pub fn assign(&mut self, name: &str, value: InterpreterLiteral) -> Result<(), &'static str> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else {
            if let Some(parent) = &self.parent {
                parent.borrow_mut().assign(name, value)
            } else {
                Err("Undefined variable usage.")
            }
        }
    }

    pub fn assign_at(me: &Rc<RefCell<Environment>>, distance: usize, name: &str, value: InterpreterLiteral) -> Result<(), &'static str> {
        Environment::ancestor(me, distance).borrow_mut().assign(name, value)
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        for (key, value) in &self.values {
            println!("[{}] -> {}", key, value);
        }
        if let Some(parent) = &self.parent {
            parent.borrow().dump();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_get() {
        let mut env = Environment::init();
        env.define(&"A".to_string(), InterpreterLiteral::Number(42.0));
        assert_eq!(InterpreterLiteral::Number(42.0), env.get("A").unwrap());
    }

    #[test]
    fn assign_existing() {
        let mut env = Environment::init();
        env.define(&"A".to_string(), InterpreterLiteral::Number(42.0));
        assert!(env.assign(&"A".to_string(), InterpreterLiteral::Nil).is_ok());
        assert_eq!(InterpreterLiteral::Nil, env.get("A").unwrap());
    }

    #[test]
    fn assign_nonexistant() {
        let mut env = Environment::init();
        assert!(env.assign("A", InterpreterLiteral::Nil).is_err());
    }

    #[test]
    fn chained_define_in_child() {
        let parent = Rc::new(RefCell::new(Environment::init()));
        let child = Rc::new(RefCell::new(Environment::init_with_parent(&parent)));
        child.borrow_mut().define("A", InterpreterLiteral::Number(42.0));
        assert!(parent.borrow().get("A").is_none());
        assert_eq!(InterpreterLiteral::Number(42.0), child.borrow().get("A").unwrap());
    }

    #[test]
    fn chained_define_in_parent() {
        let parent = Rc::new(RefCell::new(Environment::init()));
        let child = Rc::new(RefCell::new(Environment::init_with_parent(&parent)));
        parent.borrow_mut().define("A", InterpreterLiteral::Number(42.0));
        assert_eq!(InterpreterLiteral::Number(42.0), parent.borrow().get("A").unwrap());
        assert_eq!(InterpreterLiteral::Number(42.0), child.borrow().get("A").unwrap());
    }

    #[test]
    fn chained_assign_in_child() {
        let parent = Rc::new(RefCell::new(Environment::init()));
        let child = Rc::new(RefCell::new(Environment::init_with_parent(&parent)));
        parent.borrow_mut().define("A", InterpreterLiteral::Number(42.0));
        child.borrow_mut().assign("A", InterpreterLiteral::Nil).unwrap();
        assert_eq!(InterpreterLiteral::Nil, parent.borrow().get("A").unwrap());
        assert_eq!(InterpreterLiteral::Nil, child.borrow().get("A").unwrap());
    }

    #[test]
    fn chained_assign_in_parent() {
        let parent = Rc::new(RefCell::new(Environment::init()));
        let child = Rc::new(RefCell::new(Environment::init_with_parent(&parent)));
        parent.borrow_mut().define("A", InterpreterLiteral::Number(42.0));
        parent.borrow_mut().assign("A", InterpreterLiteral::Nil).unwrap();
        assert_eq!(InterpreterLiteral::Nil, parent.borrow().get("A").unwrap());
        assert_eq!(InterpreterLiteral::Nil, child.borrow().get("A").unwrap());
    }

    #[test]
    fn chained_read_from_parent() {
        let parent = Rc::new(RefCell::new(Environment::init()));
        parent.borrow_mut().define("A", InterpreterLiteral::Number(42.0));
        let child = Rc::new(RefCell::new(Environment::init_with_parent(&parent)));
        assert_eq!(InterpreterLiteral::Number(42.0), child.borrow().get("A").unwrap());
    }
}
