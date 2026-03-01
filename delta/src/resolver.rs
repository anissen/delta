use crate::{
    diagnostics::Diagnostics,
    errors::{Error, ResolutionError},
    expressions::Expr,
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

    fn resolve_expr(&mut self, expression: &'a Expr) {
        match expression {
            Expr::Identifier { name } => (),
            Expr::Context { name } => (),
            Expr::ContextIdentifier { context, name } => (),
            Expr::Grouping(expr) => self.resolve_expr(expr),
            Expr::Value { value, token } => (),
            Expr::Call { name, args } => (),
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
            Expr::Block { exprs } => exprs.iter().for_each(|expr| self.resolve_expr(expr)),
            Expr::Is { expr, arms } => (),
            Expr::Query {
                include_components,
                exclude_components,
                expr,
            } => (),
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

                ()
            }
            Expr::Create { token, arguments } => (),
            Expr::Destroy { token, argument } => (),
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
