use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::bytecodes::ByteCode;
use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{self, BinaryOperator, Expr, IsArmPattern, UnaryOperator, ValueType};
use crate::program::Context;
use crate::tokens::{Span, Token, TokenKind};

pub enum Type {
    TEMP_error,
    Boolean,
    Integer,
    Float,
    String,
    Function,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Type::TEMP_error => "TEMP",
            Type::Boolean => "boolean",
            Type::Integer => "integer",
            Type::Float => "float",
            Type::String => "string",
            Type::Function => "function(...)",
        };
        write!(f, "{}", type_name)
    }
}

pub fn type_check<'a>(
    expressions: &'a Vec<Expr>,
    context: &'a Context<'a>,
) -> Result<(), Diagnostics> {
    let mut typer = Typer::new(context);
    let result = typer.type_exprs(expressions);
    if !typer.diagnostics.has_errors() {
        Ok(())
    } else {
        Err(typer.diagnostics.clone())
    }
}

pub struct Typer<'a> {
    context: &'a Context<'a>,
    diagnostics: Diagnostics,
}

impl<'a> Typer<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Self {
            context,
            diagnostics: Diagnostics::new(),
        }
    }

    pub fn type_exprs(&mut self, expressions: &'a Vec<Expr>) -> () {
        for expr in expressions {
            self.type_expr(expr);
        }
    }

    pub fn type_expr(&mut self, expression: &'a Expr) -> Type {
        match expression {
            Expr::Value { value } => match value {
                ValueType::Boolean(_) => Type::Boolean,
                ValueType::Integer(_) => Type::Integer,
                ValueType::Float(_) => Type::Float,
                ValueType::String(_) => Type::String,
                ValueType::Function {
                    slash,
                    params,
                    expr,
                } => Type::Function,
            },

            Expr::Binary {
                left,
                operator,
                _token,
                right,
            } => self.type_binary(left, operator, _token, right),

            _ => Type::TEMP_error,
        }
    }

    fn type_binary(
        &mut self,
        left: &'a Expr,
        operator: &BinaryOperator,
        _token: &Token,
        right: &'a Expr,
    ) -> Type {
        match operator {
            BinaryOperator::Addition => {
                match self.type_expr(left) {
                    Type::Integer => (),
                    left_type => self.diagnostics.add_error(Message::new(
                        format!(
                            "Expected left part of + to be an integer but was {}",
                            left_type
                        ),
                        _token.position.clone(),
                    )),
                }

                match self.type_expr(right) {
                    Type::Integer => (),
                    right_type => self.diagnostics.add_error(Message::new(
                        format!(
                            "Expected right part of + to be an integer but was {}",
                            right_type
                        ),
                        _token.position.clone(),
                    )),
                }

                Type::Integer
            }
            _ => Type::TEMP_error,
        }
    }
}
