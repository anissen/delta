use std::collections::HashMap;
use std::fmt;

use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{
    BinaryOperator, Expr, IsArmPattern, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;
use crate::tokens::{Span, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
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

#[derive(Debug, Clone)]
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
                let function_type = env.identifiers.get_mut(name).unwrap();
                match function_type.clone() {
                    Type::Function {
                        parameters,
                        return_type,
                    } => {
                        for (index, arg) in args.iter().enumerate() {
                            let parameter = parameters[index].clone();
                            let position = positions[index].clone();
                            self.expect_type(arg, *parameter, &position, env, diagnostics);
                        }

                        *return_type.clone()
                    }
                    Type::None => Type::None,
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

            Expr::Unary {
                operator,
                _token,
                expr,
            } => match operator {
                UnaryOperator::Negation => self.type_expr(expr, env, diagnostics),
                UnaryOperator::Not => {
                    self.expect_type(expr, Type::Boolean, &_token.position, env, diagnostics);
                    Type::Boolean
                }
            },

            Expr::Grouping(expr) => self.type_expr(expr, env, diagnostics),

            Expr::Is { expr, arms } => {
                let is_type = self.type_expr(expr, env, diagnostics);
                let mut return_type = None;

                // TODO(anissen): Add positions here
                let no_position = Span { column: 0, line: 0 };
                for arm in arms {
                    // Check that arm pattern types match expr type
                    match &arm.pattern {
                        IsArmPattern::Expression(expr) => {
                            self.expect_type(
                                &expr,
                                is_type.clone(),
                                &no_position,
                                env,
                                diagnostics,
                            );
                        }

                        IsArmPattern::Capture {
                            identifier,
                            condition,
                        } => {
                            let mut new_env = env.clone();
                            new_env
                                .identifiers
                                .insert(identifier.lexeme.clone(), is_type.clone());
                            if let Some(condition) = condition {
                                self.expect_type(
                                    &condition,
                                    Type::Boolean,
                                    &no_position,
                                    &mut new_env,
                                    diagnostics,
                                );
                            }
                        }

                        IsArmPattern::Default => (),
                    }

                    // Check that return types of each arm matches
                    if let Some(return_type) = return_type.clone() {
                        self.expect_type(&arm.block, return_type, &no_position, env, diagnostics);
                    } else {
                        return_type = Some(self.type_expr(&arm.block, env, diagnostics));
                    }
                }

                return_type.unwrap()
            }
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
            BinaryOperator::IntegerOperation(_) | BinaryOperator::IntegerComparison(_) => {
                self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
                Type::Integer
            }

            BinaryOperator::FloatOperation(_) | BinaryOperator::FloatComparison(_) => {
                self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
                Type::Float
            }

            BinaryOperator::Equality(_) => {
                let left_type = self.type_expr(left, env, diagnostics);
                if left_type != Type::None {
                    self.expect_type(right, left_type, &_token.position, env, diagnostics);
                } else {
                    let right_type = self.type_expr(right, env, diagnostics);
                    self.expect_type(left, right_type, &_token.position, env, diagnostics);
                }
                Type::Boolean
            }

            BinaryOperator::BooleanOperation(_) => {
                self.expect_type(left, Type::Boolean, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Boolean, &_token.position, env, diagnostics);
                Type::Boolean
            }

            BinaryOperator::StringOperation(string_operations) => {
                match string_operations {
                    StringOperations::StringConcat => {
                        // TODO(anissen): Check types
                        Type::String
                    }
                }
            }
        }
    }

    fn error(&self, message: String, position: Span, diagnostics: &mut Diagnostics) {
        diagnostics.add_error(Message::new(message, position.clone()))
    }

    fn expect_type(
        &mut self,
        expression: &'a Expr,
        expected_type: Type,
        position: &Span,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) {
        let actual_type = self.type_expr(expression, env, diagnostics);
        if actual_type != Type::None && actual_type != expected_type {
            self.error(
                format!("Expected {} but found {}", expected_type, actual_type),
                position.clone(),
                diagnostics,
            );
        }

        if let Expr::Identifier { name } = expression {
            let typ = env.identifiers.get(&name.lexeme).unwrap();
            if typ == &Type::None {
                env.identifiers.insert(name.lexeme.clone(), expected_type);
            }
        }
    }
}
