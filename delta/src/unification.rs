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
    Tag { name: String },
    // TagUnion {  }
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
    Union {
        types: Box<Vec<UnificationType>>,
        has_wildcard: bool,
    },
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
                Type::Tag { name } => {
                    let fields = generics
                        .iter()
                        .map(|field| field.to_string())
                        .collect::<Vec<String>>();
                    if fields.is_empty() {
                        name
                    } else {
                        &format!("{}({})", name, fields.join(", "))
                    }
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
            Self::Union {
                types,
                has_wildcard,
            } => {
                let mut types_str = types
                    .iter()
                    .map(|typ| typ.to_string())
                    .collect::<Vec<String>>();
                if *has_wildcard {
                    types_str.push("*".to_string())
                }
                &format!("[{}]", types_str.join(", "))
            }
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
            UnificationType::Union {
                types,
                has_wildcard,
            } => {
                let types = types
                    .iter()
                    .map(|typ| typ.substitute(substitutions))
                    .collect::<Vec<UnificationType>>();
                UnificationType::Union {
                    types: Box::new(types),
                    has_wildcard: *has_wildcard,
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
                    && *substitution != UnificationType::Variable(v)
                {
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
            UnificationType::Union {
                types,
                has_wildcard,
            } => {
                for typ in *types {
                    if self.occurs_in(typ.clone(), substitutions) {
                        return true;
                    }
                }

                false
            }
        }
    }
}

fn unification_type_name(ty: &UnificationType) -> String {
    match ty {
        UnificationType::Constructor { token, .. } => format!("Constructor({})", token.lexeme),
        UnificationType::Union {
            types,
            has_wildcard,
        } => format!("Union({:?}, wildcard: {})", types, has_wildcard),
        UnificationType::Variable(v) => format!("Variable({})", v),
    }
}

pub fn unify(
    left: &UnificationType,
    right: &UnificationType,
    at: Option<&Token>,
    substitutions: &mut HashMap<TypeVariable, UnificationType>,
    diagnostics: &mut Diagnostics,
) {
    println!(
        "Unifying {} and {}",
        unification_type_name(left),
        unification_type_name(right)
    );
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
            // if let Type::Tag { name: tag_name1 } = name1
            //     && let Type::Tag { name: tag_name2 } = name2
            // {
            //     substitutions.insert(k, v)
            //     UnificationType::Union(vec![left, right])
            // }

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
                if let UnificationType::Constructor {
                    typ: Type::Tag { name: tag_name1 },
                    ..
                } = left
                    && let UnificationType::Constructor {
                        typ: Type::Tag { name: tag_name2 },
                        ..
                    } = &substitution
                {
                    // substitutions.insert(
                    //     v,
                    //     UnificationType::Union(Box::new(vec![left.clone(), right.clone()])),
                    // );
                }

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
        (
            UnificationType::Constructor {
                typ: name1,
                generics: generics1,
                token: token1,
            },
            UnificationType::Union {
                types,
                has_wildcard,
            },
        ) => {
            let mut mismatch_token = None;
            let has_match = types.iter().any(|union_type| match union_type {
                UnificationType::Constructor {
                    typ: name2,
                    generics: generics2,
                    token: token2,
                } => {
                    mismatch_token = Some(token2).cloned();
                    println!("*************************");
                    dbg!(&name1, &name2, &generics1, &generics2);
                    if name1 != *name2 || generics1.len() != generics2.len() {
                        return false;
                    }

                    for (left, right) in zip(&generics1, generics2) {
                        unify(&left, &right, at, substitutions, diagnostics);
                    }
                    true
                }
                UnificationType::Variable(v) => {
                    let m = substitutions.get(&v).cloned();
                    match m {
                        Some(ref substitution) => match substitution {
                            UnificationType::Constructor {
                                typ: name2,
                                generics: generics2,
                                token: token2,
                            } => {
                                mismatch_token = Some(token2).cloned();
                                println!("*!!!!************************");
                                dbg!(&name1, &name2, &generics1, &generics2);
                                if name1 != *name2 || generics1.len() != generics2.len() {
                                    return false;
                                }

                                for (left, right) in zip(&generics1, generics2) {
                                    unify(&left, &right, at, substitutions, diagnostics);
                                }
                                true
                            }
                            _ => todo!(),
                        },
                        None => todo!(),
                    }
                }
                UnificationType::Union {
                    types,
                    has_wildcard,
                } => {
                    todo!();
                }
            });
            if !has_match && !has_wildcard {
                diagnostics.add_error(Error::TypeMismatch {
                    expected: right.substitute(substitutions),
                    got: left.substitute(substitutions),
                    declared_at: token1.clone(),
                    provided_at: mismatch_token.unwrap_or(token1),
                    mismatch_at: at.cloned(),
                });
            }

            // for (left, right) in zip(generics1, generics2) {
            //     unify(&left, &right, at, substitutions, diagnostics);
            // }
        }
        (
            UnificationType::Union {
                types,
                has_wildcard,
            },
            UnificationType::Constructor {
                typ,
                generics,
                token,
            },
        ) => {
            // todo!();
        }
        (
            UnificationType::Union {
                types: types1,
                has_wildcard: has_wildcard1,
            },
            UnificationType::Union {
                types: types2,
                has_wildcard: has_wildcard2,
            },
        ) => {
            todo!();
            // check if each set of types is able to unify
            // if let Some(first) = types1.first() {
            //     types1
            //         .iter()
            //         .for_each(|t| unify(first, t, at, substitutions, diagnostics));
            // }
            // if let Some(first) = types2.first() {
            //     types2
            //         .iter()
            //         .for_each(|t| unify(first, t, at, substitutions, diagnostics));
            // }

            // for (left, right) in zip(types1.iter(), types2.iter()) {
            //     unify(left, right, at, substitutions, diagnostics);
            // }
        }
    }
}
