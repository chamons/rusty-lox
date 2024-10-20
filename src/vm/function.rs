use crate::bytecode::Chunk;

#[derive(Debug, Default)]
pub struct Function {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl Function {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_name(name: String) -> Self {
        Self {
            name: Some(name),
            ..Default::default()
        }
    }

    pub fn new_script(chunk: Chunk) -> Function {
        Function { arity: 0, chunk, name: None }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            f.write_fmt(format_args!("Function {name}"))?;
        } else {
            f.write_str("<script>")?;
        }
        Ok(())
    }
}
