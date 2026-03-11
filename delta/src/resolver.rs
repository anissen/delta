use crate::{
    diagnostics::Diagnostics,
    errors::{Error, ResolutionError},
    expressions::{Expr, IsArmPattern},
    program::Context,
    tokens::Token,
};

pub struct Resolver<'a> {
    context: &'a Context<'a>, // TODO(anissen): Check against shadowing variables and functions defined in the context
    diagnostics: &'a mut Diagnostics,
    component_names: Vec<Token>, // TODO(anissen): Component meta data needs to be a more complex structure
}

impl<'a> Resolver<'a> {
    fn new(context: &'a Context<'a>, diagnostics: &'a mut Diagnostics) -> Self {
        Self {
            context,
            diagnostics,
            component_names: Vec::new(),
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
            Expr::Identifier { name } => (),

            Expr::Context { name } => (),

            Expr::ContextIdentifier { context, name } => (),

            Expr::Grouping(expr) => self.resolve_expr(expr),

            Expr::Value { value, token } => (),

            Expr::Call { name, args } => {
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
                operator,
                token,
                expr,
            } => self.resolve_expr(expr),

            Expr::Binary {
                left,
                operator,
                token,
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
                include_components,
                exclude_components,
                expr,
            } => {
                // TODO(anissen): Resolve include/exclude components

                self.resolve_expr(expr);
            }

            Expr::ComponentDefinition { name, properties } => {
                if name.lexeme == "Entity" {
                    self.error(ResolutionError::BuiltinComponentRedefined { name: name.clone() });
                }

                if let Some(definition) = self
                    .component_names
                    .iter()
                    .find(|token| name.lexeme == token.lexeme)
                {
                    self.error(ResolutionError::ComponentRedefined {
                        name: name.clone(),
                        definition: definition.clone(),
                    });
                }
                self.component_names.push(name.clone());

                // TODO(anissen): Also check properties
            }
            Expr::Create { token, arguments } => self.resolve_expr(arguments),

            Expr::Destroy { token, argument } => self.resolve_expr(argument),

            Expr::FieldAccess {
                identifier,
                field_name,
            } => (),
        }
    }

    fn error(&mut self, err: ResolutionError) {
        self.diagnostics.add_error(Error::ResolutionErr(err));
    }
}

pub fn resolve<'a>(expression: &'a Expr, context: &'a Context<'a>, diagnostics: &mut Diagnostics) {
    let mut resolver = Resolver::new(context, diagnostics);
    resolver.resolve_expr(expression);
}
