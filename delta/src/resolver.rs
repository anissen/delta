use crate::{
    diagnostics::Diagnostics,
    environment::{ComponentMetadata, Environment},
    errors::{Error, ResolutionError},
    expressions::{Expr, IsArmPattern},
    program::Context,
    tokens::Token,
};

pub struct Resolver<'a> {
    context: &'a Context<'a>, // TODO(anissen): Check against shadowing variables and functions defined in the context
    diagnostics: &'a mut Diagnostics,
    environment: &'a mut Environment,
}

impl<'a> Resolver<'a> {
    fn new(
        context: &'a Context<'a>,
        environment: &'a mut Environment,
        diagnostics: &'a mut Diagnostics,
    ) -> Self {
        Self {
            context,
            diagnostics,
            environment,
        }
    }

    fn resolve_exprs(&mut self, expressions: &'a Vec<Expr>) {
        for expression in expressions {
            self.resolve_expr(expression);
        }
    }

    // TODO(anissen): Consider having begin_scope/end_scope helper functions for scope management

    fn resolve_expr(&mut self, expression: &'a Expr) {
        match expression {
            Expr::Identifier { name: _ } => (),

            Expr::Context { name: _ } => (),

            Expr::ContextIdentifier {
                context: _,
                name: _,
            } => (),

            Expr::Value { value: _, token: _ } => (),

            Expr::Call { name: _, args } => {
                self.resolve_exprs(args);
            }

            Expr::Assignment {
                target,
                _operator,
                expr,
            } => {
                self.resolve_expr(target);
                self.resolve_expr(expr);
            }

            Expr::Unary {
                operator: _,
                token: _,
                expr,
            } => self.resolve_expr(expr),

            Expr::Binary {
                left,
                operator: _,
                token: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Block { exprs } => self.resolve_exprs(exprs),

            Expr::Is { token, expr, arms } => {
                self.resolve_expr(expr);

                if arms.is_empty() {
                    self.error(ResolutionError::IsWithoutArms {
                        token: token.clone(),
                    });
                }

                let mut default_arm: Option<Token> = None;
                for arm in arms {
                    if let Some(default_arm_token) = default_arm {
                        return match arm.pattern {
                            IsArmPattern::Default { ref token } => {
                                self.error(ResolutionError::IsWithMultipleDefaultArms {
                                    token: token.clone(),
                                    default_arm_token: default_arm_token.clone(),
                                });
                            }
                            _ => self.error(ResolutionError::UnreachableArm {
                                token: token.clone(),
                                default_arm_token: default_arm_token.clone(),
                            }),
                        };
                    }
                    match arm.pattern {
                        IsArmPattern::Default { ref token } if arm.guard.is_none() => {
                            default_arm = Some(token.clone())
                        }
                        _ => (),
                    }
                }

                // TODO(anissen): Check for multiple capture arms or arms after a capture arm
            }

            Expr::Query {
                components: _,
                expr,
            } => {
                // TODO(anissen): Resolve include/exclude components

                self.resolve_expr(expr);
            }

            Expr::ComponentDefinition { name, properties } => {
                if name.lexeme == "Entity" {
                    self.error(ResolutionError::BuiltinComponentRedefined { name: name.clone() });
                }

                if let Some(metadata) = self
                    .environment
                    .components
                    .iter()
                    .find(|(component_name, _)| **component_name == name.lexeme)
                    .map(|(_, metadata)| metadata)
                {
                    self.error(ResolutionError::ComponentRedefined {
                        name: name.clone(),
                        definition: metadata.token.clone(),
                    });
                }

                let metadata = ComponentMetadata {
                    token: name.clone(),
                    id: self.environment.components.len() as u8,
                    properties: properties.clone(),
                };
                self.environment
                    .components
                    .insert(name.lexeme.clone(), metadata);

                // TODO(anissen): Also check properties
            }
            Expr::Create {
                token: _,
                arguments,
            } => self.resolve_expr(arguments),

            Expr::Destroy { token: _, argument } => self.resolve_expr(argument),

            Expr::FieldAccess {
                identifier: _,
                field_name: _,
            } => (),
        }
    }

    fn error(&mut self, err: ResolutionError) {
        self.diagnostics.add_error(Error::ResolutionErr(err));
    }
}

pub fn resolve<'a>(
    expression: &'a Expr,
    context: &'a Context<'a>,
    environment: &'a mut Environment,
    diagnostics: &mut Diagnostics,
) {
    let mut resolver = Resolver::new(context, environment, diagnostics);
    resolver.resolve_expr(expression);
}
