use std::fmt;

use crate::tokens::Token;
use crate::unification::UnificationType;

#[derive(Debug, Clone)]
pub enum Error {
    SyntaxError {
        description: String,
        token: Token,
    },
    ParseErr {
        message: String,
        token: Token,
    },
    TypeMismatch {
        expected: UnificationType,
        got: UnificationType,
        declared_at: Token,
        provided_at: Token,
        mismatch_at: Option<Token>,
    },
    NameNotFound {
        token: Token,
    },
    TypeRedefinition {
        token: Token,
    },
    TypeNotFound {
        token: Token,
    },
    FunctionNotFound {
        name: String,
    },
    FunctionNameTooLong {
        token: Token,
    },
    FileErr(String),
    PropertyMissing {
        property_definition: Token,
        token: Token,
    },
    PropertyDuplicated {
        token: Token,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SyntaxError { description, token } => {
                write!(
                    f,
                    "Line {}.{}: Syntax error: {}",
                    token.position.line, token.position.column, description
                )
            }
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
                mismatch_at: _,
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
            Error::PropertyMissing {
                property_definition,
                token,
            } => {
                write!(
                    f,
                    "Line {}.{}: Property missing: '{}'",
                    token.position.line, token.position.column, property_definition.lexeme
                )
            }
            Error::TypeRedefinition { token } => {
                write!(
                    f,
                    "Line {}.{}: Type '{}' redefined",
                    token.position.line, token.position.column, token.lexeme
                )
            }
            Error::TypeNotFound { token } => {
                write!(
                    f,
                    "Line {}.{}: Type '{}' not found",
                    token.position.line, token.position.column, token.lexeme
                )
            }
            Error::PropertyDuplicated { token } => {
                write!(
                    f,
                    "Line {}.{}: Property '{}' is duplicated",
                    token.position.line, token.position.column, token.lexeme
                )
            }
        }
    }
}

pub trait ErrorDescription {
    fn print(&self, source: &str) -> String;
}

impl ErrorDescription for Error {
    fn print(&self, source: &str) -> String {
        match self {
            Error::SyntaxError { description, token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::ParseErr { message: _, token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::TypeMismatch {
                expected: _,
                got: _,
                declared_at,
                provided_at,
                mismatch_at,
            } => {
                let error_declared = get_error_line(source, declared_at);
                let declared_line = declared_at.position.line;

                if let Some(mismatch_at) = mismatch_at {
                    let mismatch_line = mismatch_at.position.line;
                    let provided_line = provided_at.position.line;
                    let error_mismatch = get_error_line(source, mismatch_at);
                    let error_provided = get_error_line(source, provided_at);
                    let start = "\x1b[90m";
                    let end = "\x1b[0m";
                    format!(
                        "{start}Line {mismatch_line}: Type mismatch:{end}\n{error_mismatch}\n\n{start}Line {provided_line}: Expected this type:{end}\n{error_provided}\n\n{start}Line {declared_line}: Got this type:{end}\n{error_declared}\n\n{self}"
                    )
                } else {
                    format!("{error_declared}\n{self}")
                }
            }
            Error::NameNotFound { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::FunctionNotFound { name: _ } => {
                format!("???\n{self}")
            }
            Error::FunctionNameTooLong { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::FileErr(_error_msg) => {
                format!("???\n{self}")
            }
            Error::TypeRedefinition { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::TypeNotFound { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::PropertyMissing {
                property_definition: _,
                token,
            } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
            }
            Error::PropertyDuplicated { token } => {
                let error_line = get_error_line(source, token);
                format!("{error_line}\n{self}")
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

    let line = lines[position.line - 1];
    let mut result = String::new();
    result.push_str(line);
    result.push('\n');

    // Add spaces up to the error column
    let line_whitespace_ending = line.find(|c: char| !c.is_ascii_whitespace()).unwrap_or(0);
    let original_whitespace = &line[0..line_whitespace_ending];
    result.push_str(original_whitespace);

    if position.column - 1 > original_whitespace.len() {
        let extra_spaces = &" ".repeat(position.column - original_whitespace.len() - 1);
        result.push_str(extra_spaces);
    }

    result.push_str("\x1b[33m");

    // Add the caret indicators
    result.push_str(&"^".repeat(token.lexeme.len()));
    result.push_str("\x1b[0m");

    result
}
