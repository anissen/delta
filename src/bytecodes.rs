#[derive(Debug)]
pub enum ByteCode {
    Addition,
    Division,
    GetLocalValue,
    Multiplication,
    Negation,
    Modulo,
    Equals,
    LessThan,
    LessThanEquals,
    Not,
    PushTrue,
    PushFalse,
    PushFloat,
    PushInteger,
    PushString,
    SetLocalValue,
    Subtraction,
    StringConcat,
    BooleanAnd,
    BooleanOr,
    FunctionSignature,
    FunctionStart,
    FunctionEnd, // TODO(anissen): Maybe FunctionDefinition + FunctionBodyStart + FunctionBodyEnd?
    Call,
    CallForeign,
    GetForeignValue,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
}

impl From<ByteCode> for u8 {
    fn from(c: ByteCode) -> u8 {
        c as u8
    }
}

impl TryFrom<u8> for ByteCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == ByteCode::Addition as u8 => Ok(Self::Addition),
            value if value == ByteCode::Division as u8 => Ok(Self::Division),
            value if value == ByteCode::GetLocalValue as u8 => Ok(Self::GetLocalValue),
            value if value == ByteCode::Multiplication as u8 => Ok(Self::Multiplication),
            value if value == ByteCode::Negation as u8 => Ok(Self::Negation),
            value if value == ByteCode::Modulo as u8 => Ok(Self::Modulo),
            value if value == ByteCode::Equals as u8 => Ok(Self::Equals),
            value if value == ByteCode::LessThan as u8 => Ok(Self::LessThan),
            value if value == ByteCode::LessThanEquals as u8 => Ok(Self::LessThanEquals),
            value if value == ByteCode::Not as u8 => Ok(Self::Not),
            value if value == ByteCode::PushTrue as u8 => Ok(Self::PushTrue),
            value if value == ByteCode::PushFalse as u8 => Ok(Self::PushFalse),
            value if value == ByteCode::PushFloat as u8 => Ok(Self::PushFloat),
            value if value == ByteCode::PushInteger as u8 => Ok(Self::PushInteger),
            value if value == ByteCode::PushString as u8 => Ok(Self::PushString),
            value if value == ByteCode::SetLocalValue as u8 => Ok(Self::SetLocalValue),
            value if value == ByteCode::Subtraction as u8 => Ok(Self::Subtraction),
            value if value == ByteCode::StringConcat as u8 => Ok(Self::StringConcat),
            value if value == ByteCode::BooleanAnd as u8 => Ok(Self::BooleanAnd),
            value if value == ByteCode::BooleanOr as u8 => Ok(Self::BooleanOr),
            value if value == ByteCode::FunctionSignature as u8 => Ok(Self::FunctionSignature),
            value if value == ByteCode::FunctionStart as u8 => Ok(Self::FunctionStart),
            value if value == ByteCode::FunctionEnd as u8 => Ok(Self::FunctionEnd),
            // value if value == ByteCode::ForeignFunction as u8 => Ok(Self::ForeignFunction),
            // value if value == ByteCode::FunctionSignature as u8 => Ok(Self::FunctionSignature),
            value if value == ByteCode::Call as u8 => Ok(Self::Call),
            value if value == ByteCode::CallForeign as u8 => Ok(Self::CallForeign),
            value if value == ByteCode::GetForeignValue as u8 => Ok(Self::GetForeignValue),
            value if value == ByteCode::Jump as u8 => Ok(Self::Jump),
            value if value == ByteCode::JumpIfTrue as u8 => Ok(Self::JumpIfTrue),
            value if value == ByteCode::JumpIfFalse as u8 => Ok(Self::JumpIfFalse),
            _ => Err(()),
        }
    }
}
