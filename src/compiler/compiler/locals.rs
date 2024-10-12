use crate::compiler::tokens::token::Token;

pub struct Local {
    pub token: Token,
    pub depth: u32,
    pub initialized: bool,
}
