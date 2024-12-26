use crate::tokens::Token;

#[derive(Debug)]
pub enum Expr {
    Value(String),
    Boolean(bool),
    Grouping(Box<Expr>),
    Integer(i32),
    Float(f32),
    String(String),
    Function {
        params: Vec<Token>, // TODO(anissen): Do we also need type information here?
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>, // TODO(anissen): Should arguments be named? E.g. `square value:5`.
    },
    // ForeignCall {
    //     name: String,
    //     args: Vec<Expr>,
    // },
    Assignment {
        value: String,
        _token: Token,
        expr: Box<Expr>,
    },
    Comparison {
        left: Box<Expr>,
        token: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOperator,
        _token: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        _token: Token,
        right: Box<Expr>,
    },
    Block {
        exprs: Vec<Expr>,
    },
    // TODO(anissen): Add an Error and/or Todo expression?
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
    Modulus,
    StringConcat,
}
