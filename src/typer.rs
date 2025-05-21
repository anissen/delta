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
    Unknown,
    Any, // TODO(anissen): Remove this
    Boolean,
    Integer,
    Float,
    String,
    Function {
        parameters: Vec<Box<Type>>,
        return_type: Box<Type>,
    },
}

// https://github.com/abs0luty/type_inference_in_rust/blob/main/src/main.rs

// impl Type {
//     pub fn unify(&self, ) {}
// }

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Type::Unknown => "???",
            Type::Any => "any",
            Type::Boolean => "bool",
            Type::Integer => "int",
            Type::Float => "float",
            Type::String => "string",
            Type::Function {
                parameters,
                return_type,
            } => {
                let p = parameters
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                &format!("function({}) -> {}", p, return_type)
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
}

impl<'a> Typer<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Self { context }
    }

    fn type_exprs(
        &mut self,
        expressions: &'a Vec<Expr>,
        env: &mut TypeEnvironment,
        diagnostics: &mut Diagnostics,
    ) -> Type {
        let mut typ = Type::Unknown;
        for expr in expressions {
            typ = self.type_expr(expr, env, diagnostics);
            println!("-------------------");
            println!("{expr:?}");
            println!("===> has type {typ:?}");
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
                    let mut new_env = env.clone();
                    for p in params {
                        new_env.identifiers.insert(p.lexeme.clone(), Type::Unknown);
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
                    .unwrap_or(&Type::Unknown)
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
                let function_type = env.identifiers.get(name).unwrap();
                match function_type.clone() {
                    Type::Function {
                        parameters,
                        return_type,
                    } => {
                        let mut arg_env = env.clone();
                        let arg_types = args
                            .iter()
                            .map(|arg| Box::new(self.type_expr(&arg, &mut arg_env, diagnostics)))
                            .collect();

                        let actual = Type::Function {
                            parameters: arg_types,
                            return_type: Box::new(Type::Unknown),
                        };

                        let unified = self.unify(function_type.clone(), actual.clone());
                        match unified {
                            Ok(t) => {
                                env.identifiers.insert(name.clone(), t);
                            }
                            Err(err) => {
                                self.error(
                                    format!("Expected {} but found {}", function_type, actual),
                                    positions.first().unwrap().clone(),
                                    diagnostics,
                                );
                            }
                        }

                        *return_type.clone()
                    }

                    Type::Unknown => {
                        let t = Type::Function {
                            parameters: args.iter().map(|_| Box::new(Type::Unknown)).collect(),
                            return_type: Box::new(Type::Unknown),
                        };
                        env.identifiers.insert(name.clone(), t);
                        Type::Unknown
                    }

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
                let mut is_type = self.type_expr(expr, env, diagnostics);
                let mut return_type = None;

                // TODO(anissen): Add positions here
                let no_position = Span { column: 0, line: 0 };
                for arm in arms {
                    let mut new_env = env.clone();

                    // Check that arm pattern types match expr type
                    match &arm.pattern {
                        IsArmPattern::Expression(expr) => {
                            let pattern_type = self.type_expr(expr, env, diagnostics);
                            if is_type == Type::Unknown {
                                is_type = pattern_type;
                            }

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

                    // TODO(anissen): Check for exhaustiveness

                    // Check that return types of each arm matches
                    if let Some(return_type) = return_type.clone() {
                        self.expect_type(
                            &arm.block,
                            return_type,
                            &no_position,
                            &mut new_env,
                            diagnostics,
                        );
                    } else {
                        return_type = Some(self.type_expr(&arm.block, &mut new_env, diagnostics));
                    }
                }

                if let Expr::Identifier { name } = &**expr {
                    env.identifiers.insert(name.lexeme.clone(), is_type.clone());
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
            BinaryOperator::IntegerOperation(_) => {
                self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
                Type::Integer
            }

            BinaryOperator::IntegerComparison(_) => {
                self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
                Type::Boolean
            }

            BinaryOperator::FloatOperation(_) => {
                self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
                Type::Float
            }

            BinaryOperator::FloatComparison(_) => {
                self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
                self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
                Type::Boolean
            }

            BinaryOperator::Equality(_) => {
                let left_type = self.type_expr(left, env, diagnostics);
                if left_type != Type::Unknown {
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
                        self.expect_type(left, Type::String, &_token.position, env, diagnostics);
                        self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
                        Type::String
                    }
                }
            }
        }
    }

    fn error(&self, message: String, position: Span, diagnostics: &mut Diagnostics) {
        diagnostics.add_error(Message::new(message, position.clone()))
    }

    // TODO: Move to Type impl?
    fn unify(&self, t1: Type, t2: Type) -> Result<Type, String> {
        match (&t1, &t2) {
            (Type::Any, _) => Ok(t2),
            (Type::Unknown, _) => Ok(t2),
            (
                Type::Function {
                    parameters: expected_parameters,
                    return_type: expected_return_type,
                },
                Type::Function {
                    parameters: actual_parameters,
                    return_type: actual_return_type,
                },
            ) => {
                let mut concrete_parameters = Vec::new();
                for (expected, actual) in expected_parameters.iter().zip(actual_parameters.iter()) {
                    let concrete_parameter = self.unify(*expected.clone(), *actual.clone())?;
                    concrete_parameters.push(Box::new(concrete_parameter));
                }
                let concrete_return_type =
                    self.unify(*expected_return_type.clone(), *actual_return_type.clone())?;
                Ok(Type::Function {
                    parameters: concrete_parameters,
                    return_type: Box::new(concrete_return_type),
                })
            }
            _ => {
                if t1 == t2 {
                    Ok(t1)
                } else {
                    println!("*** FAILED TO UNIFY!");
                    println!("*** t1 = {t1:?}");
                    println!("*** t2 = {t2:?}");
                    Err("failed to unify".to_string())
                }
            }
        }
    }

    // unify_types(t1, t2) -> T, type_match(t1, t2) -> bool

    // the singleton equation set { f(1,y) = f(x,2) } is a syntactic first-order unification problem
    // that has the substitution { x ↦ 1, y ↦ 2 } as its only solution.

    fn is_same_or_more_concrete(&self, expected: Type, actual: Type) -> bool {
        // let expected_concrete = self.unify(expected.clone(), actual.clone());
        match (&expected, &actual) {
            (Type::Any, _) | (Type::Unknown, _) => true,
            (
                Type::Function {
                    parameters: expected_parameters,
                    return_type: expected_return_type,
                },
                Type::Function {
                    parameters: actual_parameters,
                    return_type: actual_return_type,
                },
            ) => {
                // println!("comparing functions");
                // dbg!(&expected, &actual);
                // dbg!(&expected_parameters, &actual_parameters);
                // dbg!(&expected_return_type, &actual_return_type);
                if expected_parameters.len() != actual_parameters.len() {
                    false
                } else if !self.is_same_or_more_concrete(
                    *expected_return_type.clone(),
                    *actual_return_type.clone(),
                ) {
                    false
                } else {
                    // dbg!(&expected, &actual);
                    // dbg!(&expected_parameters, &actual_parameters);
                    for (expected, actual) in
                        expected_parameters.iter().zip(actual_parameters.iter())
                    {
                        // println!("comparing function parameters");
                        // dbg!(&expected, &actual);
                        if !self.is_same_or_more_concrete(*expected.clone(), *actual.clone()) {
                            return false;
                        }
                    }
                    true
                }
            }
            _ => expected == actual,
        }
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
        // let unified_type = self.unify(expected_type.clone(), actual_type.clone());

        if !self.is_same_or_more_concrete(expected_type.clone(), actual_type.clone()) {
            dbg!(&actual_type);
            // dbg!(&env);
            // dbg!(self.unify(expected_type.clone(), actual_type.clone()));
            self.error(
                format!("Expected {} but found {}", expected_type, actual_type),
                position.clone(),
                diagnostics,
            );
        }

        if let Expr::Identifier { name } = expression {
            let typ = env.identifiers.get(&name.lexeme).unwrap();
            if typ == &Type::Unknown {
                env.identifiers.insert(name.lexeme.clone(), expected_type);
            }
        }
    }
}
