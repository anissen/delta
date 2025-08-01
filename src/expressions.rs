use crate::tokens::Token;

#[derive(Debug)]
pub enum ValueType {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Function {
        params: Vec<Token>, // TODO(anissen): Do we also need type information here?
        expr: Box<Expr>,
    },
    Tag {
        name: Token,
        payload: Box<Option<Expr>>,
    },
}

#[derive(Debug)]
pub enum Expr {
    Identifier {
        name: Token,
    },
    Grouping(Box<Expr>),
    Value {
        value: ValueType,
        token: Token,
    },
    Call {
        name: Token,
        args: Vec<Expr>, // TODO(anissen): Should arguments be named? E.g. `square value:5`.
    },
    // ForeignCall {
    //     name: String,
    //     args: Vec<Expr>,
    // },
    Assignment {
        name: Token,
        _operator: Token,
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
    Is {
        expr: Box<Expr>,
        arms: Vec<IsArm>,
    },
    // TODO(anissen): Add an Error and/or Todo expression?
}

#[derive(Debug)]
pub struct IsGuard {
    pub token: Token,
    pub condition: Expr,
}

#[derive(Debug)]
pub struct IsArm {
    pub pattern: IsArmPattern,
    pub guard: Option<IsGuard>,
    pub block: Expr,
}

#[derive(Debug)]
pub enum IsArmPattern {
    Expression(Expr),
    Capture {
        identifier: Token,
    },
    CaptureTagPayload {
        // tag_name: Token,
        expr: Expr, /* TODO(anissen): Should this be a Value instead? */
        identifier: Token,
    },
    Default,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negation,
    Not,
}

#[derive(Debug)]
pub enum ArithmeticOperations {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulus,
}

#[derive(Debug)]
pub enum BooleanOperations {
    And,
    Or,
}

#[derive(Debug)]
pub enum StringOperations {
    StringConcat,
}

#[derive(Debug)]
pub enum EqualityOperations {
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum Comparisons {
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
}

#[derive(Debug)]
pub enum BinaryOperator {
    IntegerOperation(ArithmeticOperations),
    FloatOperation(ArithmeticOperations),
    BooleanOperation(BooleanOperations),
    StringOperation(StringOperations),
    IntegerComparison(Comparisons),
    FloatComparison(Comparisons),
    Equality(EqualityOperations),
}
