#[derive(Debug)]
pub enum ByteCode {
    PushInteger,
    PushFloat,
    Addition,
    Subtraction,
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
            value if value == ByteCode::PushInteger as u8 => Ok(Self::PushInteger),
            value if value == ByteCode::PushFloat as u8 => Ok(Self::PushFloat),
            value if value == ByteCode::Addition as u8 => Ok(Self::Addition),
            value if value == ByteCode::Subtraction as u8 => Ok(Self::Subtraction),
            _ => Err(()),
        }
    }
}
