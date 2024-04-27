use crate::tokens::Token;

#[derive(Debug)]
pub enum Expr {
    // Error(),
    Indentation(u32),
    Integer(i32),
    Float(f32),
    Unary {
        operator: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}
