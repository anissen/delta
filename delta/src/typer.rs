use std::collections::HashMap;

use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::expressions::{
    BinaryOperator, Expr, IsArmPattern, IsGuard, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;
use crate::tokens::Token;
use crate::tokens::{Position, TokenKind};
use crate::unification::{Type, TypeVariable, UnificationType, make_constructor, unify};

// https://github.com/abs0luty/type_inference_in_rust/blob/main/src/main.rs

pub fn type_check<'a>(
    expression: &'a Expr,
    context: &'a Context<'a>,
    diagnostics: &mut Diagnostics,
) {
    let mut typer = Typer::new(context, diagnostics);
    typer.type_expr(expression);
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

    fn type_expr(&mut self, expression: &'a Expr) {
        let mut environment = Environment::new();

        let no_position = Position { line: 0, column: 0 }; // TODO(anissen): Get proper position
        let no_token = Token {
            kind: TokenKind::Underscore,
            position: no_position.clone(),
            lexeme: "".to_string(),
        };
        for value in self.context.get_value_names() {
            environment.variables.insert(
                value,
                UnificationType::Constructor {
                    typ: Type::Float,
                    generics: Vec::new(),
                    token: no_token.clone(),
                },
            );
        }

        environment.variables.insert(
            "draw_circle".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        environment.variables.insert(
            "draw_text".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    make_constructor(Type::String, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Boolean, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        environment.variables.insert(
            "draw_rect".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Boolean, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        environment.variables.insert(
            "draw_image".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    make_constructor(Type::String, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Float, no_token.clone()),
                    make_constructor(Type::Boolean, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        environment.variables.insert(
            "is_key_down".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    make_constructor(Type::String, no_token.clone()),
                    make_constructor(Type::Boolean, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        let tt = UnificationType::Variable(42); // TODO(anissen): This is a horrible hack to fix the return type to the list type parameter
        environment.variables.insert(
            "get_list_element_at_index".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    UnificationType::Constructor {
                        typ: Type::List,
                        generics: vec![tt.clone()],
                        token: no_token.clone(),
                    },
                    make_constructor(Type::Integer, no_token.clone()),
                    tt,
                ],
                token: no_token.clone(),
            },
        );

        environment.variables.insert(
            "get_array_length".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    UnificationType::Constructor {
                        typ: Type::List,
                        generics: vec![UnificationType::Variable(45)],
                        token: no_token.clone(),
                    },
                    make_constructor(Type::Integer, no_token.clone()),
                ],
                token: no_token.clone(),
            },
        );

        let ttt = UnificationType::Variable(46); // TODO(anissen): Hack!
        environment.variables.insert(
            "append".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![
                    UnificationType::Constructor {
                        typ: Type::List,
                        generics: vec![ttt.clone()],
                        token: no_token.clone(),
                    },
                    ttt.clone(),
                    UnificationType::Constructor {
                        typ: Type::List,
                        generics: vec![ttt],
                        token: no_token.clone(),
                    },
                ],
                token: no_token.clone(),
            },
        );

        let tttt = UnificationType::Variable(47); // TODO(anissen): Hack!
        environment.variables.insert(
            "log".to_string(),
            UnificationType::Constructor {
                typ: Type::Function,
                generics: vec![tttt.clone(), tttt],
                token: no_token.clone(),
            },
        );
        // for function in self.context.get_function_names() {
        //     environment.variables.insert(
        //         function,
        //         UnificationType::Constructor {
        //             typ: Type::Float,
        //             generics: Vec::new(),
        //             position: noPosition.clone(),
        //         },
        //     );
        // }

        let mut context = InferenceContext::new(&mut environment, self.diagnostics);

        context.infer_type(expression);

        context.solve();
    }
}

enum Constraint {
    Eq {
        left: UnificationType,
        right: UnificationType,
        at: Option<Token>,
    },
}

#[derive(Default)]
struct Environment {
    variables: HashMap<String, UnificationType>,
    components: HashMap<
        String, /* TODO(anissen): Should be a Token */
        Vec<crate::expressions::PropertyDefinition>,
    >,
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
            at: None,
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

            Expr::ContextIdentifier { name } => {
                // TODO(anissen): Implement
                make_constructor(Type::Float, name.clone())
            }

            Expr::ComponentDefinition { name, properties } => {
                if self.environment.components.contains_key(&name.lexeme) {
                    self.diagnostics.add_error(Error::TypeRedefinition {
                        token: name.clone(),
                    });
                }

                self.environment
                    .components
                    .insert(name.lexeme.clone(), properties.clone());

                UnificationType::Constructor {
                    typ: Type::Component,
                    generics: properties
                        .iter()
                        .map(|p| make_constructor(p.type_.clone(), p.name.clone()))
                        .collect(),
                    token: name.clone(),
                }
            }

            Expr::Value { value, token } => match value {
                ValueType::Boolean(_) => make_constructor(Type::Boolean, token.clone()),
                ValueType::Integer(_) => make_constructor(Type::Integer, token.clone()),
                ValueType::Float(_) => make_constructor(Type::Float, token.clone()),
                ValueType::String(_) => make_constructor(Type::String, token.clone()),
                ValueType::Tag { name, payload } => UnificationType::Constructor {
                    typ: Type::Tag,
                    // generics: payload.iter().map(|p| self.infer_type(p)).collect(),
                    generics: Vec::new(),
                    token: token.clone(),
                },
                ValueType::List(list) => match list.first() {
                    Some(first_element) => {
                        let first_element_type = self.infer_type(first_element);
                        for index in 1..list.len() {
                            let elm = list.get(index).unwrap();
                            self.expects_type(elm, first_element_type.clone());
                        }

                        UnificationType::Constructor {
                            typ: Type::List,
                            generics: vec![first_element_type],
                            token: token.clone(),
                        }
                    }
                    None => UnificationType::Constructor {
                        typ: Type::List,
                        generics: Vec::new(),
                        token: token.clone(),
                    },
                },
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
                        token: token.clone(),
                    }
                }
                ValueType::Component { name, properties } => {
                    let mut map = HashMap::new();
                    for prop in properties {
                        if let Some(_already_declared) =
                            map.insert(prop.name.lexeme.clone(), self.infer_type(&prop.value))
                        {
                            self.diagnostics.add_error(Error::PropertyDuplicated {
                                token: prop.name.clone(),
                            });
                        }
                    }

                    if let Some(definition) = self.environment.components.get(&name.lexeme) {
                        let filled_in_properties: Vec<_> = definition
                            .iter()
                            .flat_map(|def_prop| {
                                let res = map.get(&def_prop.name.lexeme).cloned();
                                if res.is_none() {
                                    self.diagnostics.add_error(Error::PropertyMissing {
                                        property_definition: def_prop.name.clone(),
                                        token: name.clone(),
                                    });
                                }
                                res
                            })
                            .collect();

                        UnificationType::Constructor {
                            typ: Type::Component,
                            generics: filled_in_properties,
                            token: name.clone(),
                        }
                    } else {
                        self.diagnostics.add_error(Error::TypeNotFound {
                            token: name.clone(),
                        });
                        self.type_placeholder()
                    }
                }
            },

            Expr::Call { name, args } => {
                let argument_types = args
                    .iter()
                    .map(|arg| self.infer_type(arg))
                    .collect::<Vec<UnificationType>>();
                let return_type = self.type_placeholder();

                match self.environment.variables.get(&name.lexeme) {
                    Some(function_type) => self.constraints.push(Constraint::Eq {
                        left: function_type.clone(),
                        right: UnificationType::Constructor {
                            typ: Type::Function,
                            generics: [argument_types, vec![return_type.clone()]].concat(),
                            token: name.clone(),
                        },
                        at: Some(name.clone()),
                    }),
                    None => self.diagnostics.add_error(Error::FunctionNotFound {
                        name: name.lexeme.clone(),
                    }),
                }

                return_type
            }

            Expr::Assignment {
                target,
                _operator,
                expr,
            } => {
                match **target {
                    Expr::Identifier { ref name } => {
                        self.environment
                            .variables
                            .insert(name.lexeme.clone(), UnificationType::Variable(50));
                    }
                    Expr::ContextIdentifier { ref name } => {
                        self.environment
                            .variables
                            .insert(name.lexeme.clone(), UnificationType::Variable(50));
                    }
                    _ => panic!("Invalid assignment target"),
                }
                let expr_type = self.infer_type(expr);
                match **target {
                    Expr::Identifier { ref name } => {
                        self.environment
                            .variables
                            .insert(name.lexeme.clone(), expr_type.clone());
                    }
                    Expr::ContextIdentifier { ref name } => {
                        // TODO(anissen): This is not right for ContextIdentifier?!?
                        self.environment
                            .variables
                            .insert(name.lexeme.clone(), expr_type.clone());
                    }
                    _ => panic!("Invalid assignment target"),
                }
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
                token,
                right,
            } => match operator {
                BinaryOperator::IntegerOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Integer, token.clone()));
                    self.expects_type(right, make_constructor(Type::Integer, token.clone()));
                    make_constructor(Type::Integer, token.clone())
                }

                BinaryOperator::IntegerComparison(_) => {
                    self.expects_type(left, make_constructor(Type::Integer, token.clone()));
                    self.expects_type(right, make_constructor(Type::Integer, token.clone()));
                    make_constructor(Type::Boolean, token.clone())
                }

                BinaryOperator::FloatOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Float, token.clone()));
                    self.expects_type(right, make_constructor(Type::Float, token.clone()));
                    make_constructor(Type::Float, token.clone())
                }

                BinaryOperator::FloatComparison(_) => {
                    self.expects_type(left, make_constructor(Type::Float, token.clone()));
                    self.expects_type(right, make_constructor(Type::Float, token.clone()));
                    make_constructor(Type::Boolean, token.clone())
                }

                BinaryOperator::Equality(_) => {
                    let left_type = self.infer_type(left);
                    self.expects_type(right, left_type);
                    make_constructor(Type::Boolean, token.clone())
                }

                BinaryOperator::BooleanOperation(_) => {
                    self.expects_type(left, make_constructor(Type::Boolean, token.clone()));
                    self.expects_type(right, make_constructor(Type::Boolean, token.clone()));
                    make_constructor(Type::Boolean, token.clone())
                }

                BinaryOperator::StringOperation(string_operations) => {
                    match string_operations {
                        StringOperations::StringConcat => {
                            self.expects_type(left, make_constructor(Type::String, token.clone()));
                            // TODO(anissen): Implement:
                            // self.expect_type(right, Type::Any, &_token.position, env, diagnostics); // TODO(anissen): Check types
                            make_constructor(Type::String, token.clone())
                        }
                    }
                }
            },

            Expr::Unary {
                operator,
                token,
                expr,
            } => match operator {
                UnaryOperator::Negation => self.infer_type(expr),
                UnaryOperator::Not => {
                    self.expects_type(expr, make_constructor(Type::Boolean, token.clone()));
                    make_constructor(Type::Boolean, token.clone())
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

                        IsArmPattern::Capture { identifier } => {
                            self.environment
                                .variables
                                .insert(identifier.lexeme.clone(), is_type.clone());
                        }

                        IsArmPattern::CaptureTagPayload { expr, identifier } => {
                            self.expects_type(expr, is_type.clone());
                            self.expects_type(
                                expr,
                                make_constructor(Type::Tag, identifier.clone()),
                            );
                            self.environment
                                .variables
                                .insert(identifier.lexeme.clone(), is_type.clone());
                            if let Expr::Value {
                                value: ValueType::Tag { name, payload },
                                token,
                            } = expr
                                && let Some(payload_expr) = payload.as_ref()
                            {
                                let payload_type = self.infer_type(payload_expr);
                                self.environment
                                    .variables
                                    .insert(name.lexeme.clone(), payload_type.clone());
                            }
                        }

                        IsArmPattern::Default => (),
                    }

                    if let Some(IsGuard { token, condition }) = &arm.guard {
                        self.expects_type(
                            condition,
                            make_constructor(Type::Boolean, token.clone()),
                        );
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
                Constraint::Eq { left, right, at } => {
                    unify(
                        left,
                        right,
                        at.as_ref(),
                        &mut substitutions,
                        self.diagnostics,
                    );
                }
            }
        }

        substitutions
    }
}
