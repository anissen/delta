#[derive(Debug)]
pub enum ByteCode {
    IntegerAddition,
    IntegerSubtraction,
    IntegerDivision,
    IntegerMultiplication,
    IntegerModulo,
    IntegerLessThan,
    IntegerLessThanEquals,

    FloatAddition,
    FloatSubtraction,
    FloatDivision,
    FloatMultiplication,
    FloatModulo,
    FloatLessThan,
    FloatLessThanEquals,

    StringConcat,

    BooleanAnd,
    BooleanOr,

    Equals,

    Negation,
    Not,

    GetLocalValue,
    SetLocalValue,

    GetContextValue,
    SetContextValue,

    PushTrue,
    PushFalse,
    PushFloat,
    PushInteger,
    PushString,

    PushSimpleTag,
    PushTag,
    GetTagName,
    GetTagPayload,

    FunctionSignature,
    FunctionChunk,
    Function,
    Return,
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
            value if value == ByteCode::IntegerAddition as u8 => Ok(Self::IntegerAddition),
            value if value == ByteCode::IntegerSubtraction as u8 => Ok(Self::IntegerSubtraction),
            value if value == ByteCode::IntegerDivision as u8 => Ok(Self::IntegerDivision),
            value if value == ByteCode::IntegerMultiplication as u8 => {
                Ok(Self::IntegerMultiplication)
            }
            value if value == ByteCode::IntegerModulo as u8 => Ok(Self::IntegerModulo),
            value if value == ByteCode::IntegerLessThan as u8 => Ok(Self::IntegerLessThan),
            value if value == ByteCode::IntegerLessThanEquals as u8 => {
                Ok(Self::IntegerLessThanEquals)
            }

            value if value == ByteCode::FloatAddition as u8 => Ok(Self::FloatAddition),
            value if value == ByteCode::FloatSubtraction as u8 => Ok(Self::FloatSubtraction),
            value if value == ByteCode::FloatDivision as u8 => Ok(Self::FloatDivision),
            value if value == ByteCode::FloatMultiplication as u8 => Ok(Self::FloatMultiplication),
            value if value == ByteCode::FloatModulo as u8 => Ok(Self::FloatModulo),
            value if value == ByteCode::FloatLessThan as u8 => Ok(Self::FloatLessThan),
            value if value == ByteCode::FloatLessThanEquals as u8 => Ok(Self::FloatLessThanEquals),

            value if value == ByteCode::StringConcat as u8 => Ok(Self::StringConcat),

            value if value == ByteCode::BooleanAnd as u8 => Ok(Self::BooleanAnd),
            value if value == ByteCode::BooleanOr as u8 => Ok(Self::BooleanOr),

            value if value == ByteCode::Equals as u8 => Ok(Self::Equals),
            value if value == ByteCode::Negation as u8 => Ok(Self::Negation),
            value if value == ByteCode::Not as u8 => Ok(Self::Not),

            value if value == ByteCode::GetLocalValue as u8 => Ok(Self::GetLocalValue),
            value if value == ByteCode::SetLocalValue as u8 => Ok(Self::SetLocalValue),

            value if value == ByteCode::GetContextValue as u8 => Ok(Self::GetContextValue),
            value if value == ByteCode::SetContextValue as u8 => Ok(Self::SetContextValue),

            value if value == ByteCode::PushTrue as u8 => Ok(Self::PushTrue),
            value if value == ByteCode::PushFalse as u8 => Ok(Self::PushFalse),
            value if value == ByteCode::PushFloat as u8 => Ok(Self::PushFloat),
            value if value == ByteCode::PushInteger as u8 => Ok(Self::PushInteger),
            value if value == ByteCode::PushString as u8 => Ok(Self::PushString),

            value if value == ByteCode::PushSimpleTag as u8 => Ok(Self::PushSimpleTag),
            value if value == ByteCode::PushTag as u8 => Ok(Self::PushTag),
            value if value == ByteCode::GetTagName as u8 => Ok(Self::GetTagName),
            value if value == ByteCode::GetTagPayload as u8 => Ok(Self::GetTagPayload),

            value if value == ByteCode::FunctionSignature as u8 => Ok(Self::FunctionSignature),
            value if value == ByteCode::FunctionChunk as u8 => Ok(Self::FunctionChunk),
            value if value == ByteCode::Function as u8 => Ok(Self::Function),
            value if value == ByteCode::Return as u8 => Ok(Self::Return),
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
