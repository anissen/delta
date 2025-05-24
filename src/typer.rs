use std::collections::HashMap;
use std::fmt;
use std::iter::zip;

use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{
    BinaryOperator, Expr, IsArmPattern, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;

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
) -> Result<(), Diagnostics> {
    let mut typer = Typer::new(context);

    let mut diagnostics = Diagnostics::new();
    typer.type_exprs(expressions, &mut diagnostics);
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
        diagnostics: &mut Diagnostics,
    ) -> UnificationType {
        let mut environment = Environment::new();
        let mut context = InferenceContext::new(&mut environment);
        let mut last_type = None;

        for expression in expressions {
            last_type = Some(context.infer_type(expression));
        }

        context.solve(diagnostics);

        // let substitutions = context.solve();
        // println!("Substitutions: {:?}", substitutions);

        last_type.unwrap()
    }
}

type TypeVariable = usize;

#[derive(Debug, Clone, PartialEq)]
enum UnificationType {
    Constructor {
        typ: Type,
        generics: Vec<UnificationType>,
    },
    Variable(TypeVariable),
}

impl fmt::Display for UnificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Self::Constructor { typ, generics } => match typ {
                Type::Boolean => "bool",
                Type::Integer => "int",
                Type::Float => "float",
                Type::String => "string",
                Type::Function => {
                    let parameters = generics[0..generics.len() - 1]
                        .iter()
                        .map(|param| param.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                    let return_type = generics.last().unwrap();
                    &format!("function({}) -> {}", parameters, return_type)
                }
            },
            Self::Variable(i) => &format!("???#{i}"),
        };
        write!(f, "{}", type_name)
    }
}

fn make_constructor(typ: Type) -> UnificationType {
    UnificationType::Constructor {
        typ,
        generics: Vec::new(),
    }
}

impl UnificationType {
    fn substitute(
        &self,
        substitutions: &HashMap<TypeVariable, UnificationType>,
    ) -> UnificationType {
        match self {
            UnificationType::Constructor {
                typ: name,
                generics,
            } => UnificationType::Constructor {
                typ: name.clone(),
                generics: generics
                    .iter()
                    .map(|t| t.substitute(substitutions))
                    .collect(),
            },
            UnificationType::Variable(i) => {
                if let Some(t) = substitutions.get(i) {
                    t.substitute(substitutions)
                } else {
                    self.clone()
                }
            }
        }
    }

    fn occurs_in(
        &self,
        ty: UnificationType,
        substitutions: &HashMap<TypeVariable, UnificationType>,
    ) -> bool {
        match ty {
            UnificationType::Variable(v) => {
                if let Some(substitution) = substitutions.get(&v) {
                    if *substitution != UnificationType::Variable(v) {
                        return self.occurs_in(substitution.clone(), substitutions);
                    }
                }

                self == &ty
            }
            UnificationType::Constructor { generics, .. } => {
                for generic in generics {
                    if self.occurs_in(generic, substitutions) {
                        return true;
                    }
                }

                false
            }
        }
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
}

impl<'env> InferenceContext<'env> {
    fn new(environment: &'env mut Environment) -> Self {
        Self {
            constraints: Vec::new(),
            environment,
            last_type_variable_index: 0,
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
            Expr::Identifier { name } => self
                .environment
                .variables
                .get(&name.lexeme)
                .unwrap_or_else(|| panic!("unbound variable: {}", name.lexeme))
                .clone(),

            Expr::Value { value } => match value {
                ValueType::Boolean(_) => make_constructor(Type::Boolean),
                ValueType::Integer(_) => make_constructor(Type::Integer),
                ValueType::Float(_) => make_constructor(Type::Float),
                ValueType::String(_) => make_constructor(Type::String),
                ValueType::Function {
                    slash,
                    params,
                    expr,
                } => {
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

                let function_type = self.environment.variables.get(name).unwrap(); // TODO(anissen): Maybe have a callee expression instead of name?
                self.constraints.push(Constraint::Eq {
                    left: function_type.clone(),
                    right: UnificationType::Constructor {
                        typ: Type::Function,
                        generics: [argument_types, vec![return_type.clone()]].concat(),
                    },
                });

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
                    self.expects_type(left, make_constructor(Type::Integer));
                    self.expects_type(right, make_constructor(Type::Integer));
                    make_constructor(Type::Integer)
                }

                BinaryOperator::IntegerComparison(_) => {
                    self.expects_type(left, make_constructor(Type::Integer));
                    self.expects_type(right, make_constructor(Type::Integer));
                    make_constructor(Type::Boolean)
                }

                BinaryOperator::FloatOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Float));
                    self.expects_type(right, make_constructor(Type::Float));
                    make_constructor(Type::Float)
                }

                BinaryOperator::FloatComparison(_) => {
                    self.expects_type(left, make_constructor(Type::Float));
                    self.expects_type(right, make_constructor(Type::Float));
                    make_constructor(Type::Boolean)
                }

                BinaryOperator::Equality(_) => {
                    // TODO(anissen): Implement:
                    // let left_type = self.type_expr(left, env, diagnostics);
                    // if left_type != Type::Unknown {
                    //     self.expect_type(right, left_type, &_token.position, env, diagnostics);
                    // } else {
                    //     let right_type = self.type_expr(right, env, diagnostics);
                    //     self.expect_type(left, right_type, &_token.position, env, diagnostics);
                    // }
                    make_constructor(Type::Boolean)
                }

                BinaryOperator::BooleanOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Boolean));
                    self.expects_type(right, make_constructor(Type::Boolean));
                    make_constructor(Type::Boolean)
                }

                BinaryOperator::StringOperation(string_operations) => {
                    match string_operations {
                        StringOperations::StringConcat => {
                            self.expects_type(left, make_constructor(Type::String));
                            // TODO(anissen): Implement:
                            // self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
                            make_constructor(Type::String)
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
                    self.expects_type(expr, make_constructor(Type::Boolean));
                    make_constructor(Type::Boolean)
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
                                self.expects_type(condition, make_constructor(Type::Boolean));
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

    fn solve(self, diagnostics: &mut Diagnostics) -> HashMap<TypeVariable, UnificationType> {
        let mut substitutions = HashMap::new();

        for constraint in self.constraints {
            match constraint {
                Constraint::Eq { left, right } => {
                    unify(left, right, &mut substitutions, diagnostics);
                }
            }
        }

        substitutions
    }
}

fn unify(
    left: UnificationType,
    right: UnificationType,
    substitutions: &mut HashMap<TypeVariable, UnificationType>,
    diagnostics: &mut Diagnostics,
) {
    match (left.clone(), right.clone()) {
        (
            UnificationType::Constructor {
                typ: name1,
                generics: generics1,
            },
            UnificationType::Constructor {
                typ: name2,
                generics: generics2,
            },
        ) => {
            if name1 != name2 || generics1.len() != generics2.len() {
                diagnostics.add_error(Message::from_error(format!(
                    "expected {} but got {}",
                    right.substitute(substitutions),
                    left.substitute(substitutions)
                )));
            }
            // assert_eq!(
            //     name1,
            //     name2,
            //     "expected {} but got {}",
            //     right.substitute(substitutions),
            //     left.substitute(substitutions)
            // );
            // assert_eq!(generics1.len(), generics2.len());

            for (left, right) in zip(generics1, generics2) {
                unify(left, right, substitutions, diagnostics);
            }
        }
        (UnificationType::Variable(i), UnificationType::Variable(j)) if i == j => {}
        (_, UnificationType::Variable(v)) => {
            if let Some(substitution) = substitutions.get(&v) {
                unify(left, substitution.clone(), substitutions, diagnostics);
                return;
            }

            assert!(!right.occurs_in(left.clone(), substitutions));
            substitutions.insert(v, left);
        }
        (UnificationType::Variable(v), _) => {
            if let Some(substitution) = substitutions.get(&v) {
                unify(right, substitution.clone(), substitutions, diagnostics);
                return;
            }

            assert!(!left.occurs_in(right.clone(), substitutions));
            substitutions.insert(v, right);
        }
    }
}
