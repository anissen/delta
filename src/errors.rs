use crate::tokens::{Position, Token};
use crate::unification::UnificationType;

#[derive(Debug, Clone)]
pub enum Error {
    ParseError {
        message: String,
        position: Position,
    },
    TypeMismatch {
        expected: UnificationType,
        got: UnificationType,
        declared_at: Position, // TODO: Maybe Token instead?
        provided_at: Position,
    },
    NameNotFound {
        token: Token,
    },
    FunctionNotFound {
        name: String,
    },
    FunctionNameTooLong {
        token: Token,
    },
    FileError(String),
}
