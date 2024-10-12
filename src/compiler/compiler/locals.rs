use crate::compiler::tokens::token::Token;

pub struct Local {
    name: Token,
    depth: u32,
}
