use std::collections::HashMap;

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
    List(Vec<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    Identifier {
        name: Token,
    },
    ContextIdentifier {
        // context: Option<Token>,
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
        // name: Token,
        target: Box<Expr>,
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
    ComponentDefinition {
        name: Token,
        properties: Vec<PropertyDefinition>,
    },
    ComponentInitialization {
        name: Token,
        properties: Vec<PropertyDeclaration>,
    }, // ComponentDefinition(Component),
       // ComponentInitialization {
       //     name: Token,
       //     properties: HashMap<String, Expr>, // TODO(anissen): Key ought to be a Token, or the container a Vec instead
       //     // properties: Vec<PropertyDeclaration>,
       //     definition: Component, //Vec<PropertyDefinition>,
       // },
       // TODO(anissen): Add an Error and/or Todo expression?
}

// #[derive(Debug)]
// pub struct Component {
//     pub name: Token,
//     pub properties: Vec<PropertyDefinition>,
// }

#[derive(Debug)]
pub struct PropertyDefinition {
    pub name: Token,
    pub type_: crate::unification::Type, // TODO(anissen): Also needs to be able to represent complex types
}

#[derive(Debug)]
pub struct PropertyDeclaration {
    pub name: Token,
    pub value: Expr,
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
