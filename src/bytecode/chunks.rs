use std::fmt::Debug;

#[derive(Debug, PartialEq, PartialOrd)]
enum OpCode {
    Return,
    Constant(usize),
}

#[derive(PartialEq, PartialOrd)]
enum OpValue {
    Double(f64),
}

impl Debug for OpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Double(v) => write!(f, "'{}'", v),
        }
    }
}

struct Chunk {
    pub code: Vec<OpCode>,
    pub lines: Vec<usize>,
    pub values: Vec<OpValue>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: vec![],
            lines: vec![],
            values: vec![],
        }
    }

    pub fn write(&mut self, code: OpCode, line: usize) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn write_value(&mut self, value: OpValue) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        for (i, chunk) in self.code.iter().enumerate() {
            let instruction = match chunk {
                OpCode::Return => "OP_RETURN".to_string(),
                OpCode::Constant(index) => format!("OP_CONSTANT\t{} {:?}", index, self.values[*index]),
            };
            let line = if i > 0 && self.lines[i] == self.lines[i - 1] {
                "   | ".to_string()
            } else {
                format!("{:>4}", self.lines[i])
            };
            println!("{i:0>4} {line} {instruction}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_smoke_test() {
        let mut chunk = Chunk::new();
        let index = chunk.write_value(OpValue::Double(1.2));
        chunk.write(OpCode::Constant(index), 1);
        chunk.write(OpCode::Return, 1);
        assert_eq!(OpCode::Return, *chunk.code.last().unwrap());
        chunk.disassemble("main");
    }

    #[test]
    fn add_values() {
        let mut chunk = Chunk::new();
        assert_eq!(0, chunk.write_value(OpValue::Double(1.0)));
        assert_eq!(1, chunk.write_value(OpValue::Double(2.0)));
    }
}
