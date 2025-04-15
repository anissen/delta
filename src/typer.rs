use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{BinaryOperator, Expr, ValueType};
use crate::program::Context;
use crate::tokens::{Span, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    TEMP_error,
    Boolean,
    Integer,
    Float,
    String,
    Function {
        parameters: Vec<Box<Type>>,
        return_type: Box<Type>,
    },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Type::TEMP_error => "TEMP",
            Type::Boolean => "boolean",
            Type::Integer => "integer",
            Type::Float => "float",
            Type::String => "string",
            Type::Function {
                parameters,
                return_type,
            } => {
                // let param_str = parameters.iter().map(|p| p.fmt(f)?)
                // &format!(
                //     "function({:?}) -> {:?}",
                //     // parameters.iter().map(|p| p.fmt(f)),
                //     "[params]",
                //     return_type.fmt(f)?
                // )
                "function([???]) -> ?"
            }
        };
        write!(f, "{}", type_name)
    }
}

struct TypeEnvironment {
    identifiers: HashMap<String, Type>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            identifiers: HashMap::new(),
        }
    }
}

pub fn type_check<'a>(
    expressions: &'a Vec<Expr>,
    context: &'a Context<'a>,
) -> Result<(), Diagnostics> {
    let mut typer = Typer::new(context);

    let mut env = TypeEnvironment::new();
    let mut diagnostics = Diagnostics::new();
    let result = typer.type_exprs(expressions, &mut env, &mut diagnostics);
    if !diagnostics.has_errors() {
        Ok(())
    } else {
        Err(diagnostics.clone())
    }
}

pub struct Typer<'a> {
    context: &'a Context<'a>,
    // identifiers: HashMap<&'a String, Type>,
}

impl<'a> Typer<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Self {
            context,
            // identifiers: HashMap::new(),
        }
    }

    fn type_exprs(
        &mut self,
        expressions: &'a Vec<Expr>,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) -> () {
        for expr in expressions {
            self.type_expr(expr, env, diagnostics);
        }
    }

    fn type_expr(
        &mut self,
        expression: &'a Expr,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) -> Type {
        dbg!(&expression);
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
                } => {
                    let mut new_env = TypeEnvironment::new();
                    new_env.identifiers = env.identifiers.clone(); // TODO(anissen): HAAACK!
                    let result = self.type_expr(expr, &mut new_env, diagnostics);
                    let mut param_types = vec![];
                    for p in params {
                        dbg!(&p);
                        let typ = new_env.identifiers.get(&p.lexeme).unwrap().clone();
                        param_types.push(Box::new(typ));
                    }
                    Type::Function {
                        parameters: param_types,
                        return_type: Box::new(result),
                    }
                }
            },

            Expr::Identifier { name } => {
                env.identifiers.get(&name.lexeme).unwrap().clone() // TODO(anissen): This clone would be nice to avoid
            }

            Expr::Assignment {
                name,
                _operator,
                expr,
            } => {
                let expr_type = self.type_expr(expr, env, diagnostics);
                env.identifiers
                    .insert(name.lexeme.clone(), expr_type.clone()); // TODO(anissen): Fewer clones, please?
                expr_type
            }

            Expr::Call { name, args } => {
                let function_type = env.identifiers.get(name); // TODO(anissen): Should type_expr return a (type, scope) instead?

                let mut new_env = TypeEnvironment::new();
                new_env.identifiers = env.identifiers.clone(); // TODO(anissen): HAAAAAAAAAACK!

                match function_type {
                    Some(Type::Function {
                        parameters,
                        return_type,
                    }) => {
                        dbg!(&parameters);
                        for (index, arg) in args.iter().enumerate() {
                            let parameter = parameters[index].clone();
                            // TODO(anissen): Provide a position with call and args
                            let no_position = Span { line: 0, column: 0 };
                            // self.type_expr(arg, &mut new_env, diagnostics);
                            self.expect_type(
                                arg,
                                &parameter,
                                &no_position,
                                &mut new_env,
                                diagnostics,
                            );
                        }

                        *return_type.clone()
                    }
                    _ => panic!("cannot type check function call"),
                }
            }

            Expr::Binary {
                left,
                operator,
                _token,
                right,
            } => self.type_binary(left, operator, _token, right, env, diagnostics),

            _ => Type::TEMP_error,
        }
    }

    fn type_binary(
        &mut self,
        left: &'a Expr,
        operator: &BinaryOperator,
        _token: &Token,
        right: &'a Expr,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) -> Type {
        match operator {
            BinaryOperator::Addition | BinaryOperator::Multiplication => {
                self.expect_type(left, &Type::Integer, &_token.position, env, diagnostics);
                self.expect_type(right, &Type::Integer, &_token.position, env, diagnostics);
                Type::Integer
            }
            _ => Type::TEMP_error,
        }
    }

    fn error(&self, message: String, position: Span, diagnostics: &mut Diagnostics) {
        diagnostics.add_error(Message::new(message, position.clone()))
    }

    fn expect_type(
        &mut self,
        expression: &'a Expr,
        expected_type: &Type,
        position: &Span,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) {
        let actual_type = self.type_expr(expression, env, diagnostics);
        if actual_type != *expected_type {
            self.error(
                format!("Expected {} but found {}", expected_type, actual_type),
                position.clone(),
                diagnostics,
            );
        }
    }
}
