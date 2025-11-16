use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::tokens::Token;
use std::collections::HashMap;
use std::fmt::{self};
use std::iter::zip;

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Boolean,
    Integer,
    Float,
    String,
    Tag, // { name: String, argument_count: u8 },
    List,
    Function,
    Component,
}

pub type TypeVariable = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum UnificationType {
    Constructor {
        typ: Type,
        generics: Vec<UnificationType>,
        token: Token,
    },
    Variable(TypeVariable),
}

impl fmt::Display for UnificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Self::Constructor {
                typ,
                generics,
                token,
            } => match typ {
                Type::Boolean => "bool",
                Type::Integer => "int",
                Type::Float => "float",
                Type::String => "string",
                Type::Tag => {
                    // if *argument_count == 0 {
                    //     &format!("tag :{name}")
                    // } else {
                    //     &format!("tag :{name}({argument_count})")
                    // }
                    &"tag".to_string()
                }
                Type::List => {
                    if !generics.is_empty() {
                        let typ = generics.first().unwrap();
                        &format!("list[{typ}]")
                    } else {
                        "list[]"
                    }
                }
                Type::Function => {
                    let parameters = generics[0..generics.len() - 1]
                        .iter()
                        .map(|param| param.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                    let return_type = generics.last().unwrap();
                    &format!("function({parameters}) -> {return_type}")
                }
                Type::Component => {
                    let fields = generics
                        .iter()
                        .map(|field| field.to_string())
                        .collect::<Vec<String>>();
                    &format!("component {}({})", token.lexeme, fields.join(", "))
                }
            },
            Self::Variable(i) => &format!("???#{i}"),
        };
        write!(f, "{type_name}")
    }
}

pub fn make_constructor(typ: Type, token: Token) -> UnificationType {
    UnificationType::Constructor {
        typ,
        generics: Vec::new(),
        token,
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
                token,
            } => UnificationType::Constructor {
                typ: name.clone(),
                generics: generics
                    .iter()
                    .map(|t| t.substitute(substitutions))
                    .collect(),
                token: token.clone(),
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
                if let Some(substitution) = substitutions.get(&v)
                    && *substitution != UnificationType::Variable(v) {
                        return self.occurs_in(substitution.clone(), substitutions);
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

pub fn unify(
    left: &UnificationType,
    right: &UnificationType,
    at: Option<&Token>,
    substitutions: &mut HashMap<TypeVariable, UnificationType>,
    diagnostics: &mut Diagnostics,
) {
    match (left.clone(), right.clone()) {
        (
            UnificationType::Constructor {
                typ: name1,
                generics: generics1,
                token: token1,
            },
            UnificationType::Constructor {
                typ: name2,
                generics: generics2,
                token: token2,
            },
        ) => {
            if name1 != name2 || generics1.len() != generics2.len() {
                diagnostics.add_error(Error::TypeMismatch {
                    expected: right.substitute(substitutions),
                    got: left.substitute(substitutions),
                    declared_at: token1,
                    provided_at: token2,
                    mismatch_at: at.cloned(),
                });
            }

            for (left, right) in zip(generics1, generics2) {
                unify(&left, &right, at, substitutions, diagnostics);
            }
        }
        (UnificationType::Variable(i), UnificationType::Variable(j)) if i == j => {}
        (_, UnificationType::Variable(v)) => match substitutions.get(&v) {
            Some(substitution) => {
                unify(left, &substitution.clone(), at, substitutions, diagnostics);
            }
            None => {
                assert!(!right.occurs_in(left.clone(), substitutions));
                substitutions.insert(v, left.clone());
            }
        },
        (UnificationType::Variable(v), _) => match substitutions.get(&v) {
            Some(substitution) => {
                unify(right, &substitution.clone(), at, substitutions, diagnostics);
            }
            None => {
                assert!(!left.occurs_in(right.clone(), substitutions));
                substitutions.insert(v, right.clone());
            }
        },
    }
}
