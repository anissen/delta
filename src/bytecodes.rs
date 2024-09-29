#[derive(Debug)]
pub enum ByteCode {
    Addition,
    Division,
    GetLocalValue,
    Multiplication,
    Negation, // TODO(anissen): Rename all (here e.g. op_negate)?
    Not,
    PushBoolean, // TODO(anissen): Should be split into PushTrue + PushFalse
    PushFloat,
    PushInteger,
    SetLocalValue,
    Subtraction,
    FunctionStart,
    FunctionEnd, // TODO(anissen): Maybe FunctionDefinition + FunctionBodyStart + FunctionBodyEnd?
    // FunctionSignature,
    Call,
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
            value if value == ByteCode::Not as u8 => Ok(Self::Not),
            value if value == ByteCode::PushBoolean as u8 => Ok(Self::PushBoolean),
            value if value == ByteCode::PushFloat as u8 => Ok(Self::PushFloat),
            value if value == ByteCode::PushInteger as u8 => Ok(Self::PushInteger),
            value if value == ByteCode::SetLocalValue as u8 => Ok(Self::SetLocalValue),
            value if value == ByteCode::Subtraction as u8 => Ok(Self::Subtraction),
            value if value == ByteCode::FunctionStart as u8 => Ok(Self::FunctionStart),
            value if value == ByteCode::FunctionEnd as u8 => Ok(Self::FunctionEnd),
            // value if value == ByteCode::FunctionSignature as u8 => Ok(Self::FunctionSignature),
            value if value == ByteCode::Call as u8 => Ok(Self::Call),
            _ => Err(()),
        }
    }
}
