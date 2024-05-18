use crate::tokens::Token;

#[derive(Debug)]
pub enum Expr {
    Comment(String),  // TODO(anissen): Remove this
    Variable(String), // TODO(anissen): Rename this (the value is not variable)
    Integer(i32),
    Float(f32),
    Assignment {
        variable: String, // TODO(anissen): Rename
        expr: Box<Expr>,
    },
    Unary {
        operator: Token, // TODO(anissen): Maybe make this a unary-specific operator type
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token, // TODO(anissen): Maybe make this a binary-specific operator type
        right: Box<Expr>,
    },
}
