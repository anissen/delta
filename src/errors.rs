use std::fmt;

use crate::tokens::Token;
use crate::unification::UnificationType;

#[derive(Debug, Clone)]
pub enum Error {
    ParseErr {
        message: String,
        token: Token,
    },
    TypeMismatch {
        expected: UnificationType,
        got: UnificationType,
        declared_at: Token,
        provided_at: Token,
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
            Error::ParseErr { message, token } => {
                write!(
                    f,
                    "Line {}.{}: Parse error: {}",
                    token.position.line, token.position.column, message
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
                declared_at.position.line, declared_at.position.column, expected, got
            ),
            Error::NameNotFound { token } => {
                write!(
                    f,
                    "Line {}.{}: Name not found in scope: {}",
                    token.position.line, token.position.column, token.lexeme
                )
            }
            Error::FunctionNotFound { name } => {
                write!(f, "Function not found: {name}")
            }
            Error::FunctionNameTooLong { token } => {
                write!(f, "Function name too long; at {:?}", token.position)
            }
            Error::FileErr(error_msg) => write!(f, "File error: {error_msg}"),
        }
    }
}

pub trait ErrorDescription {
    fn print(&self, source: &str) -> String;
}

impl ErrorDescription for Error {
    fn print(&self, source: &str) -> String {
        match self {
            Error::ParseErr { message, token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::TypeMismatch {
                expected,
                got,
                declared_at,
                provided_at,
            } => {
                let error_line = get_error_line(source, declared_at);
                format!("{error_line}\n{self}")
            }
            Error::NameNotFound { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::FunctionNotFound { name } => {
                format!("???\n{self}")
            }
            Error::FunctionNameTooLong { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::FileErr(error_msg) => {
                format!("???\n{self}")
            }
        }
    }
}

fn get_error_line(source: &str, token: &Token) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let position = &token.position;
    if position.line == 0 || position.line > lines.len() {
        return String::new();
    }

    let line = lines[position.line - 1].replace('\t', " ");
    let mut result = String::new();
    result.push_str(&line);
    result.push('\n');

    // Add spaces up to the error column
    result.push_str(&" ".repeat(position.column - 1));

    // Add the caret indicators
    result.push_str(&"^".repeat(token.lexeme.len()));

    result
}
