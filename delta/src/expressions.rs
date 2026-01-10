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
        payload: Option<Box<Expr>>,
    },
    List(Vec<Expr>),
    Component {
        name: Token,
        properties: Vec<PropertyDeclaration>,
    },
}

#[derive(Debug)]
pub enum Expr {
    Identifier {
        name: Token,
    },
    Context {
        name: Token,
    },
    ContextIdentifier {
        context: Token,
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
    Query {
        include_components: Vec<NamedType>, // TODO(anissen): Should this be Vec<Expr> instead?
        exclude_components: Vec<Token>,
        expr: Box<Expr>,
    },
    ComponentDefinition {
        name: Token,
        properties: Vec<PropertyDefinition>,
    },
    Create {
        token: Token,
        arguments: Box<Expr>,
    },
    FieldAccess {
        identifier: Token,
        field_name: Token,
    },
    // TODO(anissen): Add an Error and/or Todo expression?
}

#[derive(Debug)]
pub struct NamedType {
    pub type_: Token,
    pub name: Token,
}

#[derive(Debug, Clone)]
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
    Capture { identifier: Token },
    CaptureTagPayload { tag_name: Token, identifier: Token },
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
