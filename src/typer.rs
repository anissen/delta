use std::collections::HashMap;
use std::fmt;
use std::iter::zip;

use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{
    BinaryOperator, Expr, IsArmPattern, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;
use crate::tokens::{Span, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    // Unknown,
    // Any, // TODO(anissen): Remove this
    Boolean,
    Integer,
    Float,
    String,
    Function,
}

// https://github.com/abs0luty/type_inference_in_rust/blob/main/src/main.rs

// impl Type {
//     pub fn unify(&self, ) {}
// }

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
    ) -> UnificationType {
        let mut environment = Environment::new();
        let mut context = InferenceContext::new(&mut environment);
        let mut last_type = None;

        for expression in expressions {
            last_type = Some(context.infer_type(&expression));
            // context.solve();
        }
        let substitutions = context.solve();
        println!("Substitutions: {:?}", substitutions);
        // println!("Substituted Type: {:?}", ty.substitute(&substitutions));
        // println!("{:?}", ty.substitute(&substitutions));

        // let mut typ = Type::Unknown;
        // for expr in expressions {
        //     typ = self.type_expr(expr, env, diagnostics);
        //     println!("-------------------");
        //     println!("{expr:?}");
        //     println!("===> has type {typ:?}");
        // }
        // typ

        last_type.unwrap()
    }

    // fn type_expr(
    //     &mut self,
    //     expression: &'a Expr,
    //     env: &mut TypeEnvironment,
    //     diagnostics: &mut Diagnostics,
    // ) -> Type {
    //     match expression {
    //         Expr::Value { value } => match value {
    //             ValueType::Boolean(_) => Type::Boolean,
    //             ValueType::Integer(_) => Type::Integer,
    //             ValueType::Float(_) => Type::Float,
    //             ValueType::String(_) => Type::String,
    //             ValueType::Function {
    //                 slash,
    //                 params,
    //                 expr,
    //             } => {
    //                 let mut new_env = env.clone();
    //                 for p in params {
    //                     new_env.identifiers.insert(p.lexeme.clone(), Type::Unknown);
    //                 }
    //                 let result = self.type_expr(expr, &mut new_env, diagnostics);
    //                 let mut param_types = vec![];
    //                 for p in params {
    //                     let typ = new_env.identifiers.get(&p.lexeme).unwrap().clone();
    //                     param_types.push(Box::new(typ));
    //                 }
    //                 Type::Function {
    //                     parameters: param_types,
    //                     return_type: Box::new(result),
    //                 }
    //             }
    //         },

    //         Expr::Identifier { name } => {
    //             env.identifiers
    //                 .get(&name.lexeme)
    //                 .unwrap_or(&Type::Unknown)
    //                 .clone() // TODO(anissen): This clone would be nice to avoid
    //         }

    //         Expr::Assignment {
    //             name,
    //             _operator,
    //             expr,
    //         } => {
    //             let expr_type = self.type_expr(expr, env, diagnostics);
    //             env.identifiers
    //                 .insert(name.lexeme.clone(), expr_type.clone()); // TODO(anissen): Fewer clones, please?
    //             expr_type
    //         }

    //         Expr::Block { exprs } => self.type_exprs(exprs, env, diagnostics),

    //         Expr::Call {
    //             name,
    //             args,
    //             positions,
    //         } => {
    //             let function_type = env.identifiers.get(name).unwrap();
    //             match function_type.clone() {
    //                 Type::Function {
    //                     parameters,
    //                     return_type,
    //                 } => {
    //                     let mut arg_env = env.clone();
    //                     let arg_types = args
    //                         .iter()
    //                         .map(|arg| Box::new(self.type_expr(&arg, &mut arg_env, diagnostics)))
    //                         .collect();

    //                     let actual = Type::Function {
    //                         parameters: arg_types,
    //                         return_type: Box::new(Type::Unknown),
    //                     };

    //                     let unified = self.unify(function_type.clone(), actual.clone());
    //                     match unified {
    //                         Ok(t) => {
    //                             env.identifiers.insert(name.clone(), t);
    //                         }
    //                         Err(err) => {
    //                             self.error(
    //                                 format!("Expected {} but found {}", function_type, actual),
    //                                 positions.first().unwrap().clone(),
    //                                 diagnostics,
    //                             );
    //                         }
    //                     }

    //                     *return_type.clone()
    //                 }

    //                 Type::Unknown => {
    //                     let t = Type::Function {
    //                         parameters: args.iter().map(|_| Box::new(Type::Unknown)).collect(),
    //                         return_type: Box::new(Type::Unknown),
    //                     };
    //                     env.identifiers.insert(name.clone(), t);
    //                     Type::Unknown
    //                 }

    //                 _ => {
    //                     dbg!(&function_type);
    //                     panic!("cannot type check function call")
    //                 }
    //             }
    //         }

    //         Expr::Binary {
    //             left,
    //             operator,
    //             _token,
    //             right,
    //         } => self.type_binary(left, operator, _token, right, env, diagnostics),

    //         Expr::Unary {
    //             operator,
    //             _token,
    //             expr,
    //         } => match operator {
    //             UnaryOperator::Negation => self.type_expr(expr, env, diagnostics),
    //             UnaryOperator::Not => {
    //                 self.expect_type(expr, Type::Boolean, &_token.position, env, diagnostics);
    //                 Type::Boolean
    //             }
    //         },

    //         Expr::Grouping(expr) => self.type_expr(expr, env, diagnostics),

    //         Expr::Is { expr, arms } => {
    //             let mut is_type = self.type_expr(expr, env, diagnostics);
    //             let mut return_type = None;

    //             // TODO(anissen): Add positions here
    //             let no_position = Span { column: 0, line: 0 };
    //             for arm in arms {
    //                 let mut new_env = env.clone();

    //                 // Check that arm pattern types match expr type
    //                 match &arm.pattern {
    //                     IsArmPattern::Expression(expr) => {
    //                         let pattern_type = self.type_expr(expr, env, diagnostics);
    //                         if is_type == Type::Unknown {
    //                             is_type = pattern_type;
    //                         }

    //                         self.expect_type(
    //                             &expr,
    //                             is_type.clone(),
    //                             &no_position,
    //                             env,
    //                             diagnostics,
    //                         );
    //                     }

    //                     IsArmPattern::Capture {
    //                         identifier,
    //                         condition,
    //                     } => {
    //                         new_env
    //                             .identifiers
    //                             .insert(identifier.lexeme.clone(), is_type.clone());
    //                         if let Some(condition) = condition {
    //                             self.expect_type(
    //                                 &condition,
    //                                 Type::Boolean,
    //                                 &no_position,
    //                                 &mut new_env,
    //                                 diagnostics,
    //                             );
    //                         }
    //                     }

    //                     IsArmPattern::Default => (),
    //                 }

    //                 // TODO(anissen): Check for exhaustiveness

    //                 // Check that return types of each arm matches
    //                 if let Some(return_type) = return_type.clone() {
    //                     self.expect_type(
    //                         &arm.block,
    //                         return_type,
    //                         &no_position,
    //                         &mut new_env,
    //                         diagnostics,
    //                     );
    //                 } else {
    //                     return_type = Some(self.type_expr(&arm.block, &mut new_env, diagnostics));
    //                 }
    //             }

    //             if let Expr::Identifier { name } = &**expr {
    //                 env.identifiers.insert(name.lexeme.clone(), is_type.clone());
    //             }

    //             return_type.unwrap()
    //         }
    //     }
    // }

    // fn type_binary(
    //     &mut self,
    //     left: &'a Expr,
    //     operator: &BinaryOperator,
    //     _token: &Token,
    //     right: &'a Expr,
    //     env: &mut TypeEnvironment,
    //     diagnostics: &mut Diagnostics,
    // ) -> Type {
    //     match operator {
    //         BinaryOperator::IntegerOperation(_) => {
    //             self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
    //             self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
    //             Type::Integer
    //         }

    //         BinaryOperator::IntegerComparison(_) => {
    //             self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
    //             self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
    //             Type::Boolean
    //         }

    //         BinaryOperator::FloatOperation(_) => {
    //             self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
    //             self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
    //             Type::Float
    //         }

    //         BinaryOperator::FloatComparison(_) => {
    //             self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
    //             self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
    //             Type::Boolean
    //         }

    //         BinaryOperator::Equality(_) => {
    //             let left_type = self.type_expr(left, env, diagnostics);
    //             if left_type != Type::Unknown {
    //                 self.expect_type(right, left_type, &_token.position, env, diagnostics);
    //             } else {
    //                 let right_type = self.type_expr(right, env, diagnostics);
    //                 self.expect_type(left, right_type, &_token.position, env, diagnostics);
    //             }
    //             Type::Boolean
    //         }

    //         BinaryOperator::BooleanOperation(_) => {
    //             self.expect_type(left, Type::Boolean, &_token.position, env, diagnostics);
    //             self.expect_type(right, Type::Boolean, &_token.position, env, diagnostics);
    //             Type::Boolean
    //         }

    //         BinaryOperator::StringOperation(string_operations) => {
    //             match string_operations {
    //                 StringOperations::StringConcat => {
    //                     self.expect_type(left, Type::String, &_token.position, env, diagnostics);
    //                     self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
    //                     Type::String
    //                 }
    //             }
    //         }
    //     }
    // }

    fn error(&self, message: String, position: Span, diagnostics: &mut Diagnostics) {
        diagnostics.add_error(Message::new(message, position.clone()))
    }

    // // TODO: Move to Type impl?
    // fn unify(&self, t1: Type, t2: Type) -> Result<Type, String> {
    //     match (&t1, &t2) {
    //         // (Type::Any, _) => Ok(t2),
    //         // (Type::Unknown, _) => Ok(t2),
    //         (
    //             Type::Function {
    //                 parameters: expected_parameters,
    //                 return_type: expected_return_type,
    //             },
    //             Type::Function {
    //                 parameters: actual_parameters,
    //                 return_type: actual_return_type,
    //             },
    //         ) => {
    //             let mut concrete_parameters = Vec::new();
    //             for (expected, actual) in expected_parameters.iter().zip(actual_parameters.iter()) {
    //                 let concrete_parameter = self.unify(*expected.clone(), *actual.clone())?;
    //                 concrete_parameters.push(Box::new(concrete_parameter));
    //             }
    //             let concrete_return_type =
    //                 self.unify(*expected_return_type.clone(), *actual_return_type.clone())?;
    //             Ok(Type::Function {
    //                 parameters: concrete_parameters,
    //                 return_type: Box::new(concrete_return_type),
    //             })
    //         }
    //         _ => {
    //             if t1 == t2 {
    //                 Ok(t1)
    //             } else {
    //                 println!("*** FAILED TO UNIFY!");
    //                 println!("*** t1 = {t1:?}");
    //                 println!("*** t2 = {t2:?}");
    //                 Err("failed to unify".to_string())
    //             }
    //         }
    //     }
    // }

    // unify_types(t1, t2) -> T, type_match(t1, t2) -> bool

    // the singleton equation set { f(1,y) = f(x,2) } is a syntactic first-order unification problem
    // that has the substitution { x ↦ 1, y ↦ 2 } as its only solution.

    // fn is_same_or_more_concrete(&self, expected: Type, actual: Type) -> bool {
    //     // let expected_concrete = self.unify(expected.clone(), actual.clone());
    //     match (&expected, &actual) {
    //         (Type::Any, _) | (Type::Unknown, _) => true,
    //         (
    //             Type::Function {
    //                 parameters: expected_parameters,
    //                 return_type: expected_return_type,
    //             },
    //             Type::Function {
    //                 parameters: actual_parameters,
    //                 return_type: actual_return_type,
    //             },
    //         ) => {
    //             // println!("comparing functions");
    //             // dbg!(&expected, &actual);
    //             // dbg!(&expected_parameters, &actual_parameters);
    //             // dbg!(&expected_return_type, &actual_return_type);
    //             if expected_parameters.len() != actual_parameters.len() {
    //                 false
    //             } else if !self.is_same_or_more_concrete(
    //                 *expected_return_type.clone(),
    //                 *actual_return_type.clone(),
    //             ) {
    //                 false
    //             } else {
    //                 // dbg!(&expected, &actual);
    //                 // dbg!(&expected_parameters, &actual_parameters);
    //                 for (expected, actual) in
    //                     expected_parameters.iter().zip(actual_parameters.iter())
    //                 {
    //                     // println!("comparing function parameters");
    //                     // dbg!(&expected, &actual);
    //                     if !self.is_same_or_more_concrete(*expected.clone(), *actual.clone()) {
    //                         return false;
    //                     }
    //                 }
    //                 true
    //             }
    //         }
    //         _ => expected == actual,
    //     }
    // }

    // fn expect_type(
    //     &mut self,
    //     expression: &'a Expr,
    //     expected_type: Type,
    //     position: &Span,
    //     env: &mut TypeEnvironment,
    //     diagnostics: &mut Diagnostics,
    // ) {
    //     let actual_type = self.type_expr(expression, env, diagnostics);
    //     // let unified_type = self.unify(expected_type.clone(), actual_type.clone());

    //     if !self.is_same_or_more_concrete(expected_type.clone(), actual_type.clone()) {
    //         dbg!(&actual_type);
    //         // dbg!(&env);
    //         // dbg!(self.unify(expected_type.clone(), actual_type.clone()));
    //         self.error(
    //             format!("Expected {} but found {}", expected_type, actual_type),
    //             position.clone(),
    //             diagnostics,
    //         );
    //     }

    //     if let Expr::Identifier { name } = expression {
    //         let typ = env.identifiers.get(&name.lexeme).unwrap();
    //         if typ == &Type::Unknown {
    //             env.identifiers.insert(name.lexeme.clone(), expected_type);
    //         }
    //     }
    // }
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
        println!("Expects {actual_type} to be {expected_type}");
        self.constraints.push(Constraint::Eq {
            left: actual_type,
            right: expected_type,
        });
        // unify(actual_type, expected_type, );
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

                _ => panic!("sadf"), // BinaryOperator::IntegerComparison(_) => {
                                     //     self.expect_type(left, Type::Integer, &_token.position, env, diagnostics);
                                     //     self.expect_type(right, Type::Integer, &_token.position, env, diagnostics);
                                     //     Type::Boolean
                                     // }

                                     // BinaryOperator::FloatOperation(_) => {
                                     //     self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
                                     //     self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
                                     //     Type::Float
                                     // }

                                     // BinaryOperator::FloatComparison(_) => {
                                     //     self.expect_type(left, Type::Float, &_token.position, env, diagnostics);
                                     //     self.expect_type(right, Type::Float, &_token.position, env, diagnostics);
                                     //     Type::Boolean
                                     // }

                                     // BinaryOperator::Equality(_) => {
                                     //     let left_type = self.type_expr(left, env, diagnostics);
                                     //     if left_type != Type::Unknown {
                                     //         self.expect_type(right, left_type, &_token.position, env, diagnostics);
                                     //     } else {
                                     //         let right_type = self.type_expr(right, env, diagnostics);
                                     //         self.expect_type(left, right_type, &_token.position, env, diagnostics);
                                     //     }
                                     //     Type::Boolean
                                     // }

                                     // BinaryOperator::BooleanOperation(_) => {
                                     //     self.expect_type(left, Type::Boolean, &_token.position, env, diagnostics);
                                     //     self.expect_type(right, Type::Boolean, &_token.position, env, diagnostics);
                                     //     Type::Boolean
                                     // }

                                     // BinaryOperator::StringOperation(string_operations) => {
                                     //     match string_operations {
                                     //         StringOperations::StringConcat => {
                                     //             self.expect_type(
                                     //                 left,
                                     //                 Type::String,
                                     //                 &_token.position,
                                     //                 env,
                                     //                 diagnostics,
                                     //             );
                                     //             self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
                                     //             Type::String
                                     //         }
                                     //     }
                                     // }
            },

            t => {
                println!("Type not handled: {:?}", t);
                panic!("not handled")
            }
        }
    }

    fn solve(self) -> HashMap<TypeVariable, UnificationType> {
        let mut substitutions = HashMap::new();

        for constraint in self.constraints {
            match constraint {
                Constraint::Eq { left, right } => {
                    unify(left, right, &mut substitutions);
                }
            }
        }

        substitutions
    }
}

fn make_constructor(typ: Type) -> UnificationType {
    UnificationType::Constructor {
        typ,
        generics: Vec::new(),
    }
}

fn unify(
    left: UnificationType,
    right: UnificationType,
    substitutions: &mut HashMap<TypeVariable, UnificationType>,
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
            // assert_eq!(name1, name2, "expected {name2} but got {name1}");
            assert_eq!(name1, name2, "expected {right} but got {left}");
            assert_eq!(generics1.len(), generics2.len());

            for (left, right) in zip(generics1, generics2) {
                unify(left, right, substitutions);
            }
        }
        (UnificationType::Variable(i), UnificationType::Variable(j)) if i == j => {}
        (_, UnificationType::Variable(v)) => {
            if let Some(substitution) = substitutions.get(&v) {
                unify(left, substitution.clone(), substitutions);
                return;
            }

            assert!(!right.occurs_in(left.clone(), substitutions));
            substitutions.insert(v, left);
        }
        (UnificationType::Variable(v), _) => {
            if let Some(substitution) = substitutions.get(&v) {
                unify(right, substitution.clone(), substitutions);
                return;
            }

            assert!(!left.occurs_in(right.clone(), substitutions));
            substitutions.insert(v, right);
        }
    }
}

// fn make_tconst(typ: &str, generics: Vec<UnificationType>) -> UnificationType {
//     UnificationType::Constructor {
//         name: typ.to_string(),
//         generics,
//     }
// }

// fn make_tvar(i: usize) -> Type {
//     Type::Variable(TypeVariable(i))
// }

// fn main() {
//     let mut environment = Environment::new();

//     /*

//     add = \v
//         \x
//             v + x

//     a | (a | add)
//     */
//     environment.variables.insert(
//         "add".to_owned(),
//         make_tconst(
//             "Function",
//             vec![
//                 make_tconst("int", vec![]),
//                 make_tconst(
//                     "Function",
//                     vec![make_tconst("int", vec![]), make_tconst("int", vec![])],
//                 ),
//             ],
//         ),
//     );

//     let mut context = InferenceContext::new(&mut environment);

//     let expression = Expression::Lambda {
//         parameter: "a".to_string(),
//         value: Box::new(Expression::Apply {
//             callee: Box::new(Expression::Apply {
//                 callee: Box::new(Expression::Variable {
//                     name: "add".to_string(),
//                 }),
//                 argument: Box::new(Expression::Variable {
//                     name: "a".to_string(),
//                 }),
//             }),
//             argument: Box::new(Expression::Variable {
//                 name: "a".to_string(),
//             }),
//         }),
//     };
//     let ty = context.infer_type(&expression);
//     let substitutions = context.solve();

//     println!("{:?}", ty.substitute(&substitutions));
// }
