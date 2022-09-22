use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedString(u32);

pub struct Interner {
    map: HashMap<String, InternedString>,
    vec: Vec<String>,
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            map: HashMap::new(),
            vec: vec![],
        }
    }

    pub fn intern(&mut self, name: &str) -> InternedString {
        if let Some(&idx) = self.map.get(name) {
            return idx;
        }
        let idx = InternedString(self.map.len() as u32);
        self.map.insert(name.to_owned(), idx);
        self.vec.push(name.to_owned());
        idx
    }

    pub fn lookup(&self, idx: InternedString) -> &str {
        self.vec[idx.0 as usize].as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_smoke() {
        let mut interner = Interner::new();
        let first = interner.intern("asdf");
        let second = interner.intern("fdsa");
        let third = interner.intern("asdf");
        assert_ne!(first, second);
        assert_eq!(first, third);
        assert_eq!("asdf", interner.lookup(first));
        assert_eq!("fdsa", interner.lookup(second));
        assert_eq!("asdf", interner.lookup(third));
    }
}
