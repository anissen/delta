use std::collections::HashMap;

use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::expressions::{
    BinaryOperator, Expr, IsArmPattern, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;
use crate::unification::{make_constructor, unify, TypeVariable, UnificationType};

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Boolean,
    Integer,
    Float,
    String,
    Function,
}

// https://github.com/abs0luty/type_inference_in_rust/blob/main/src/main.rs

pub fn type_check<'a>(
    expressions: &'a Vec<Expr>,
    context: &'a Context<'a>,
    diagnostics: &mut Diagnostics,
) {
    let mut typer = Typer::new(context, diagnostics);
    typer.type_exprs(expressions);
}

pub struct Typer<'a> {
    context: &'a Context<'a>,
    diagnostics: &'a mut Diagnostics,
}

impl<'a> Typer<'a> {
    fn new(context: &'a Context<'a>, diagnostics: &'a mut Diagnostics) -> Self {
        Self {
            context,
            diagnostics,
        }
    }

    fn type_exprs(&mut self, expressions: &'a Vec<Expr>) {
        let mut environment = Environment::new();
        let mut context = InferenceContext::new(&mut environment, self.diagnostics);

        for expression in expressions {
            context.infer_type(expression);
        }

        context.solve();
    }
}

enum Constraint {
    Eq {
        left: UnificationType,
        right: UnificationType,
    },
}

#[derive(Default)]
struct Environment {
    variables: HashMap<String, UnificationType>,
}

impl Environment {
    fn new() -> Self {
        Self::default()
    }
}

struct InferenceContext<'env> {
    constraints: Vec<Constraint>,
    environment: &'env mut Environment,
    last_type_variable_index: usize,
    diagnostics: &'env mut Diagnostics,
}

impl<'env> InferenceContext<'env> {
    fn new(environment: &'env mut Environment, diagnostics: &'env mut Diagnostics) -> Self {
        Self {
            constraints: Vec::new(),
            environment,
            last_type_variable_index: 0,
            diagnostics,
        }
    }

    fn fresh_type_variable(&mut self) -> TypeVariable {
        self.last_type_variable_index += 1;
        self.last_type_variable_index
    }

    fn type_placeholder(&mut self) -> UnificationType {
        UnificationType::Variable(self.fresh_type_variable())
    }

    fn expects_type(&mut self, expression: &Expr, expected_type: UnificationType) {
        let actual_type = self.infer_type(expression);
        self.constraints.push(Constraint::Eq {
            left: actual_type,
            right: expected_type,
        });
    }

    fn infer_type(&mut self, expression: &Expr) -> UnificationType {
        match expression {
            Expr::Identifier { name } => match self.environment.variables.get(&name.lexeme) {
                Some(value) => value.clone(),
                None => {
                    self.diagnostics.add_error(Error::NameNotFound {
                        token: name.clone(),
                    });
                    self.type_placeholder()
                }
            },

            Expr::Value { value, token } => match value {
                ValueType::Boolean(_) => make_constructor(Type::Boolean, token.position.clone()),
                ValueType::Integer(_) => make_constructor(Type::Integer, token.position.clone()),
                ValueType::Float(_) => make_constructor(Type::Float, token.position.clone()),
                ValueType::String(_) => make_constructor(Type::String, token.position.clone()),
                ValueType::Function { params, expr } => {
                    let param_types = params
                        .iter()
                        .map(|param| {
                            let parameter_type = self.type_placeholder();
                            self.environment
                                .variables
                                .insert(param.lexeme.clone(), parameter_type.clone());
                            parameter_type
                        })
                        .collect::<Vec<UnificationType>>();

                    let value_type = self.infer_type(expr);

                    UnificationType::Constructor {
                        typ: Type::Function,
                        generics: [param_types, vec![value_type]].concat(),
                        position: token.position.clone(),
                    }
                }
            },

            Expr::Call {
                name,
                args,
                positions,
            } => {
                let argument_types = args
                    .iter()
                    .map(|arg| self.infer_type(arg))
                    .collect::<Vec<UnificationType>>();
                let return_type = self.type_placeholder();

                match self.environment.variables.get(name) {
                    Some(function_type) => {
                        self.constraints.push(Constraint::Eq {
                            left: function_type.clone(),
                            right: UnificationType::Constructor {
                                typ: Type::Function,
                                generics: [argument_types, vec![return_type.clone()]].concat(),
                                position: positions.first().unwrap().clone(), // TODO(anissen): Is this right?
                            },
                        })
                    }
                    None => self
                        .diagnostics
                        .add_error(Error::FunctionNotFound { name: name.clone() }),
                }

                return_type
            }

            Expr::Assignment {
                name,
                _operator,
                expr,
            } => {
                let expr_type = self.infer_type(expr);
                self.environment
                    .variables
                    .insert(name.lexeme.clone(), expr_type.clone());
                expr_type
            }

            Expr::Block { exprs } => exprs
                .iter()
                .map(|expr| self.infer_type(expr))
                .last()
                .unwrap(),

            Expr::Binary {
                left,
                operator,
                _token,
                right,
            } => match operator {
                BinaryOperator::IntegerOperation(_) => {
                    self.expects_type(
                        left,
                        make_constructor(Type::Integer, _token.position.clone()),
                    );
                    self.expects_type(
                        right,
                        make_constructor(Type::Integer, _token.position.clone()),
                    );
                    make_constructor(Type::Integer, _token.position.clone())
                }

                BinaryOperator::IntegerComparison(_) => {
                    self.expects_type(
                        left,
                        make_constructor(Type::Integer, _token.position.clone()),
                    );
                    self.expects_type(
                        right,
                        make_constructor(Type::Integer, _token.position.clone()),
                    );
                    make_constructor(Type::Boolean, _token.position.clone())
                }

                BinaryOperator::FloatOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Float, _token.position.clone()));
                    self.expects_type(
                        right,
                        make_constructor(Type::Float, _token.position.clone()),
                    );
                    make_constructor(Type::Float, _token.position.clone())
                }

                BinaryOperator::FloatComparison(_) => {
                    self.expects_type(left, make_constructor(Type::Float, _token.position.clone()));
                    self.expects_type(
                        right,
                        make_constructor(Type::Float, _token.position.clone()),
                    );
                    make_constructor(Type::Boolean, _token.position.clone())
                }

                BinaryOperator::Equality(_) => {
                    let left_type = self.infer_type(left);
                    self.expects_type(right, left_type);
                    make_constructor(Type::Boolean, _token.position.clone())
                }

                BinaryOperator::BooleanOperation(_) => {
                    self.expects_type(
                        left,
                        make_constructor(Type::Boolean, _token.position.clone()),
                    );
                    self.expects_type(
                        right,
                        make_constructor(Type::Boolean, _token.position.clone()),
                    );
                    make_constructor(Type::Boolean, _token.position.clone())
                }

                BinaryOperator::StringOperation(string_operations) => {
                    match string_operations {
                        StringOperations::StringConcat => {
                            self.expects_type(
                                left,
                                make_constructor(Type::String, _token.position.clone()),
                            );
                            // TODO(anissen): Implement:
                            // self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
                            make_constructor(Type::String, _token.position.clone())
                        }
                    }
                }
            },

            Expr::Unary {
                operator,
                _token,
                expr,
            } => match operator {
                UnaryOperator::Negation => self.infer_type(expr),
                UnaryOperator::Not => {
                    self.expects_type(
                        expr,
                        make_constructor(Type::Boolean, _token.position.clone()),
                    );
                    make_constructor(Type::Boolean, _token.position.clone())
                }
            },

            Expr::Grouping(expr) => self.infer_type(expr),

            Expr::Is { expr, arms } => {
                let is_type = self.infer_type(expr);
                let mut return_type = None;

                // TODO(anissen): Add positions here
                for arm in arms {
                    // Check that arm pattern types match expr type
                    match &arm.pattern {
                        IsArmPattern::Expression(expr) => {
                            self.expects_type(expr, is_type.clone());
                        }

                        IsArmPattern::Capture {
                            identifier,
                            condition,
                        } => {
                            self.environment
                                .variables
                                .insert(identifier.lexeme.clone(), is_type.clone());
                            if let Some(condition) = condition {
                                self.expects_type(
                                    condition,
                                    make_constructor(Type::Boolean, identifier.position.clone()),
                                );
                            }
                        }

                        IsArmPattern::Default => (),
                    }

                    // TODO(anissen): Check for exhaustiveness

                    // Check that return types of each arm matches
                    if let Some(return_type) = return_type.clone() {
                        self.expects_type(&arm.block, return_type);
                    } else {
                        return_type = Some(self.infer_type(&arm.block));
                    }
                }

                if let Expr::Identifier { name } = &**expr {
                    self.environment
                        .variables
                        .insert(name.lexeme.clone(), is_type.clone());
                }

                return_type.unwrap()
            }
        }
    }

    fn solve(&mut self) -> HashMap<TypeVariable, UnificationType> {
        let mut substitutions = HashMap::new();

        for constraint in &self.constraints {
            match constraint {
                Constraint::Eq { left, right } => {
                    unify(left, right, &mut substitutions, self.diagnostics);
                }
            }
        }

        substitutions
    }
}
