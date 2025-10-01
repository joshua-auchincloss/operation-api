use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder, PartialEq)]
pub struct TypeDef {
    pub comment: String,
    pub ident: Ident,
    pub ty: Type,
}

impl FromInner for TypeDef {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut ty = None;
        let mut comment = String::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::ident | Rule::name => ident = Some(Ident::from_inner(Pairs::single(pair))?),
                Rule::typ => ty = Some(Type::from_inner(pair.into_inner())?),
                Rule::eq => {},
                Rule::singleline_comment | Rule::multiline_comment => {
                    comment += &take_comment(Pairs::single(pair));
                },
                rule => panic!("{rule:#?}"),
            }
        }

        if ident.is_none() || ty.is_none() {
            return Err(crate::Error::defs::<Self, _>([Rule::ident, Rule::typ]));
        }

        Ok(Self {
            comment,
            ident: ident.unwrap(),
            ty: ty.unwrap(),
        })
    }
}

impl Commentable for TypeDef {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

#[derive(Debug, Clone, PartialEq)]

pub enum Type {
    Builtin(Builtin),
    Ident(Ident),
    Union(Box<UnionType>),
    Enum(EnumDef),
    Array(Box<Type>),
}

impl Type {
    pub fn builtin(value: Builtin) -> Self {
        Self::Builtin(value)
    }

    pub fn ident<I: Into<Ident>>(value: I) -> Self {
        Self::Ident(value.into())
    }

    pub fn union(value: UnionType) -> Self {
        Self::Union(Box::new(value))
    }

    pub fn array(value: Type) -> Self {
        Self::Array(Box::new(value))
    }
}

impl Type {
    /// recursively flattens all union types into a single `Type`.
    /// 1) this is a union, returns a new union with all nested unions flattened.
    /// 2) it isnt a union, we return self
    pub fn resolve_union(self) -> Self {
        match self {
            Type::Union(union) => {
                let mut types = Vec::new();
                for t in union.types {
                    match t {
                        Type::Union(inner_union) if inner_union.kind == union.kind => {
                            // flatten nested unions of the same kind
                            types.extend(
                                inner_union
                                    .types
                                    .into_iter()
                                    .map(|t| t.resolve_union()),
                            );
                        },
                        other => types.push(other.resolve_union()),
                    }
                }
                if types.len() == 1 {
                    types.into_iter().next().unwrap()
                } else {
                    Type::Union(Box::new(UnionType {
                        types,
                        kind: union.kind,
                    }))
                }
            },
            other => other,
        }
    }
}

impl FromInner for Type {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut root = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::builtin => {
                    root = Some(Self::builtin(Builtin::from_inner(Pairs::single(pair))?));
                },
                Rule::type_operand => {
                    root = Some(
                        Self::union(UnionType::from_inner(Pairs::single(pair))?).resolve_union(),
                    );
                },
                Rule::ident => root = Some(Self::ident(Ident::from_inner(Pairs::single(pair))?)),
                Rule::typ | Rule::singular_type => {
                    root = Some(Self::from_inner(pair.into_inner())?.resolve_union());
                },
                Rule::array => {
                    root = Some(Self::array(
                        root.expect("arrays must be defined after type declaration"),
                    ))
                },
                rule => panic!("{rule:#?}"),
            }
        }

        match root {
            Some(ty) => Ok(ty),
            None => {
                Err(crate::Error::defs::<Self, _>([
                    Rule::builtin,
                    Rule::type_operand,
                    Rule::oneof,
                    Rule::ident,
                    Rule::typ,
                    Rule::singular_type,
                    Rule::array,
                ]))
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flatten_simple_union() {
        let union = Type::Union(Box::new(UnionType {
            types: vec![Type::Builtin(Builtin::I32), Type::Builtin(Builtin::F32)],
            kind: UnionKind::Or,
        }));

        let resolved = union.clone().resolve_union();
        assert_eq!(resolved, union);
    }

    #[test]
    fn test_flatten_nested_union_same_kind() {
        let nested_union = Type::Union(Box::new(UnionType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::Union(Box::new(UnionType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: UnionKind::Or,
                })),
            ],
            kind: UnionKind::Or,
        }));

        let resolved = nested_union.resolve_union();
        let expected = Type::Union(Box::new(UnionType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::Builtin(Builtin::F32),
                Type::Builtin(Builtin::Bool),
            ],
            kind: UnionKind::Or,
        }));

        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_flatten_nested_union_different_kind() {
        let nested_union = Type::Union(Box::new(UnionType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::Union(Box::new(UnionType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: UnionKind::And,
                })),
            ],
            kind: UnionKind::Or,
        }));

        let resolved = nested_union.resolve_union();
        let expected = Type::Union(Box::new(UnionType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::Union(Box::new(UnionType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: UnionKind::And,
                })),
            ],
            kind: UnionKind::Or,
        }));

        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_flatten_single_type_union() {
        let union = Type::Union(Box::new(UnionType {
            types: vec![Type::Builtin(Builtin::I32)],
            kind: UnionKind::Or,
        }));

        let resolved = union.resolve_union();
        assert_eq!(resolved, Type::Builtin(Builtin::I32));
    }

    #[test]
    fn test_flatten_non_union_type() {
        let ty = Type::Builtin(Builtin::I32);
        let resolved = ty.clone().resolve_union();
        assert_eq!(resolved, ty);
    }
}
