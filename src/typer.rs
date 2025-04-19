use std::collections::HashMap;
use std::fmt;

use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{BinaryOperator, Expr, ValueType};
use crate::program::Context;
use crate::tokens::{Span, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    TEMP_error,
    None,
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
            Type::None => "???",
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
    ) -> Type {
        let mut typ = Type::None;
        for expr in expressions {
            typ = self.type_expr(expr, env, diagnostics);
        }
        typ
    }

    fn type_expr(
        &mut self,
        expression: &'a Expr,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) -> Type {
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
                    for p in params {
                        new_env.identifiers.insert(p.lexeme.clone(), Type::None);
                    }
                    let result = self.type_expr(expr, &mut new_env, diagnostics);
                    let mut param_types = vec![];
                    for p in params {
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
                env.identifiers
                    .get(&name.lexeme)
                    .unwrap_or(&Type::None)
                    .clone() // TODO(anissen): This clone would be nice to avoid
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

            Expr::Block { exprs } => self.type_exprs(exprs, env, diagnostics),

            Expr::Call {
                name,
                args,
                positions,
            } => {
                let function_type = env.identifiers.get(name); // TODO(anissen): Should type_expr return a (type, scope) instead?

                let mut new_env = TypeEnvironment::new();
                new_env.identifiers = env.identifiers.clone(); // TODO(anissen): HAAAAAAAAAACK!

                match function_type {
                    Some(Type::Function {
                        parameters,
                        return_type,
                    }) => {
                        for (index, arg) in args.iter().enumerate() {
                            let parameter = parameters[index].clone();
                            let position = positions[index].clone();
                            // TODO(anissen): Provide a position with call and args
                            // self.type_expr(arg, &mut new_env, diagnostics);
                            self.expect_type(arg, &parameter, &position, &mut new_env, diagnostics);
                        }

                        *return_type.clone()
                    }
                    Some(Type::None) => Type::None,
                    _ => {
                        dbg!(&function_type);
                        panic!("cannot type check function call")
                    }
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
            BinaryOperator::IntegerOperation(_) => {
                // if left or right is an identifier w. no type, assign integer to it
                self.expect_type(left, &Type::Integer, &_token.position, env, diagnostics);
                if let Expr::Identifier { name } = left {
                    let typ = env.identifiers.get(&name.lexeme).unwrap();
                    if typ == &Type::None {
                        env.identifiers.insert(name.lexeme.clone(), Type::Integer);
                    }
                }
                self.expect_type(right, &Type::Integer, &_token.position, env, diagnostics);
                if let Expr::Identifier { name } = right {
                    let typ = env.identifiers.get(&name.lexeme).unwrap();
                    if typ == &Type::None {
                        env.identifiers.insert(name.lexeme.clone(), Type::Integer);
                    }
                }
                Type::Integer
            }

            BinaryOperator::FloatOperation(_) => {
                // if left or right is an identifier w. no type, assign integer to it
                self.expect_type(left, &Type::Float, &_token.position, env, diagnostics);
                if let Expr::Identifier { name } = left {
                    let typ = env.identifiers.get(&name.lexeme).unwrap();
                    if typ == &Type::None {
                        env.identifiers.insert(name.lexeme.clone(), Type::Float);
                    }
                }
                self.expect_type(right, &Type::Float, &_token.position, env, diagnostics);
                if let Expr::Identifier { name } = right {
                    let typ = env.identifiers.get(&name.lexeme).unwrap();
                    if typ == &Type::None {
                        env.identifiers.insert(name.lexeme.clone(), Type::Float);
                    }
                }
                Type::Float
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
        if actual_type != Type::None && actual_type != *expected_type {
            self.error(
                format!("Expected {} but found {}", expected_type, actual_type),
                position.clone(),
                diagnostics,
            );
        }
    }
}
