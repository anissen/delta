use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::tokens::Position;
use std::collections::HashMap;
use std::fmt;
use std::iter::zip;

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Boolean,
    Integer,
    Float,
    String,
    Function,
}

pub type TypeVariable = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum UnificationType {
    Constructor {
        typ: Type,
        generics: Vec<UnificationType>,
        position: Position,
    },
    Variable(TypeVariable),
}

impl fmt::Display for UnificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Self::Constructor {
                typ,
                generics,
                position,
            } => match typ {
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

pub fn make_constructor(typ: Type, position: Position) -> UnificationType {
    UnificationType::Constructor {
        typ,
        generics: Vec::new(),
        position,
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
                position,
            } => UnificationType::Constructor {
                typ: name.clone(),
                generics: generics
                    .iter()
                    .map(|t| t.substitute(substitutions))
                    .collect(),
                position: position.clone(),
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

pub fn unify(
    left: &UnificationType,
    right: &UnificationType,
    substitutions: &mut HashMap<TypeVariable, UnificationType>,
    diagnostics: &mut Diagnostics,
) {
    match (left.clone(), right.clone()) {
        (
            UnificationType::Constructor {
                typ: name1,
                generics: generics1,
                position: position1,
            },
            UnificationType::Constructor {
                typ: name2,
                generics: generics2,
                position: position2,
            },
        ) => {
            if name1 != name2 || generics1.len() != generics2.len() {
                diagnostics.add_error(Error::TypeMismatch {
                    expected: right.substitute(substitutions),
                    got: left.substitute(substitutions),
                    declared_at: position1,
                    provided_at: position2,
                });
            }

            for (left, right) in zip(generics1, generics2) {
                unify(&left, &right, substitutions, diagnostics);
            }
        }
        (UnificationType::Variable(i), UnificationType::Variable(j)) if i == j => {}
        (_, UnificationType::Variable(v)) => match substitutions.get(&v) {
            Some(substitution) => {
                unify(left, &substitution.clone(), substitutions, diagnostics);
            }
            None => {
                assert!(!right.occurs_in(left.clone(), substitutions));
                substitutions.insert(v, left.clone());
            }
        },
        (UnificationType::Variable(v), _) => match substitutions.get(&v) {
            Some(substitution) => {
                unify(right, &substitution.clone(), substitutions, diagnostics);
            }
            None => {
                assert!(!left.occurs_in(right.clone(), substitutions));
                substitutions.insert(v, right.clone());
            }
        },
    }
}
