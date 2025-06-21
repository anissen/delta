use std::fmt;

use crate::tokens::{Position, Token};
use crate::unification::UnificationType;

#[derive(Debug, Clone)]
pub enum Error {
    ParseErr {
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
    FileErr(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseErr { message, position } => {
                write!(
                    f,
                    "Line {}.{}: Parse error: {}",
                    position.line, position.column, message
                )
            }
            Error::TypeMismatch {
                expected,
                got,
                declared_at,
                provided_at: _,
            } => write!(
                f,
                "Line {}.{}: Expected {} but got {}.",
                declared_at.line, declared_at.column, expected, got
            ),
            Error::NameNotFound { token } => {
                write!(
                    f,
                    "Line {}.{}: Name not found in scope: {}",
                    token.position.line, token.position.column, token.lexeme
                )
            }
            Error::FunctionNotFound { name } => {
                write!(f, "Function not found: {}", name)
            }
            Error::FunctionNameTooLong { token } => {
                write!(f, "Function name too long; at {:?}", token.position)
            }
            Error::FileErr(error_msg) => write!(f, "File error: {}", error_msg),
        }
    }
}
