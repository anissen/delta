use crate::tokens::Token;

#[derive(Debug)]
pub enum Expr {
    Variable(String), // TODO(anissen): Rename this (the value is not variable)
    Boolean(bool),
    Grouping(Box<Expr>),
    Integer(i32),
    Float(f32),
    Function {
        params: Vec<Token>, // TODO(anissen): Do we also need type information here?
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>, // TODO(anissen): Should arguments be named? E.g. `square value:5`.
    },
    Assignment {
        variable: String, // TODO(anissen): Rename
        token: Token,
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
    Block {
        exprs: Vec<Expr>,
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
