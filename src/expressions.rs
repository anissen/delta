use crate::tokens::Token;

#[derive(Debug)]
pub enum Expr {
    Variable(String), // TODO(anissen): Rename this (the value is not variable)
    Boolean(bool),
    Grouping(Box<Expr>),
    Integer(i32),
    Float(f32),
    Assignment {
        variable: String, // TODO(anissen): Rename
        // token: Token,
        expr: Box<Expr>,
    },
    Unary {
        operator: UnaryOperator,
        token: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        token: Token,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negation,
    Not,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}
