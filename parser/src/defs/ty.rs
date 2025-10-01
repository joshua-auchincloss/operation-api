use crate::defs::*;

use pest::iterators::Pairs;

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder, PartialEq)]
pub struct TypeDef<V> {
    pub comment: String,
    pub ident: Ident,
    pub ty: Type<V>,
    pub meta: Vec<Meta>,
    pub version: V,
}

impl<V: Default + Clone + std::fmt::Debug + PartialEq> FromInner for TypeDef<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut ty: Option<Type<V>> = None;
        let mut comment = String::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::ident | Rule::name => {
                    let sp = Ident::from_pair_span(pair)?;
                    ident = Some(sp.value);
                },
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
            meta: Vec::new(),
            version: V::default(),
        })
    }
}

impl<V> Commentable for TypeDef<V> {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl<V: Default + Clone + std::fmt::Debug + PartialEq> FromPairSpan for TypeDef<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = TypeDef::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<V> {
    Builtin(Builtin),
    Ident(Ident),
    OneOf(Box<OneOfType<V>>),
    Enum(EnumDef<V>),
    Array(Box<Type<V>>),
}

impl<V: Clone + std::fmt::Debug + PartialEq> Type<V> {
    pub fn builtin(value: Builtin) -> Self {
        Self::Builtin(value)
    }

    pub fn ident<I: Into<Ident>>(value: I) -> Self {
        Self::Ident(value.into())
    }

    pub fn oneof(value: OneOfType<V>) -> Self {
        Self::OneOf(Box::new(value))
    }

    pub fn array(value: Type<V>) -> Self {
        Self::Array(Box::new(value))
    }
}

impl<V: Clone + std::fmt::Debug + PartialEq> Type<V> {
    pub fn resolve_union(self) -> Self {
        match self {
            Type::OneOf(union) => {
                let mut types = Vec::new();
                for t in union.types {
                    match t {
                        Type::OneOf(inner_union) if inner_union.kind == union.kind => {
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
                    Type::OneOf(Box::new(OneOfType {
                        types,
                        kind: union.kind,
                        version: union.version,
                    }))
                }
            },
            other => other,
        }
    }
}

impl<V: Clone + std::fmt::Debug + PartialEq + Default> FromInner for Type<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut root = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::builtin => {
                    root = Some(Self::builtin(Builtin::from_inner(Pairs::single(pair))?));
                },
                Rule::type_operand => {
                    root = Some(
                        Self::oneof(OneOfType::from_inner(Pairs::single(pair))?).resolve_union(),
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

impl<V: Clone + std::fmt::Debug + PartialEq + Default> FromPairSpan for Type<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = Type::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

impl TypeUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> TypeSealed {
        match self {
            Self::Array(arr) => TypeSealed::Array(Box::new(arr.seal(file_version))),
            Self::Enum(e) => TypeSealed::Enum(e.seal(file_version)),
            Self::OneOf(oneof) => TypeSealed::OneOf(Box::new(oneof.seal(file_version))),
            Self::Builtin(ty) => TypeSealed::Builtin(ty),
            Self::Ident(id) => TypeSealed::Ident(id),
        }
    }
}
pub type TypeSealed = Type<usize>;
pub type TypeUnsealed = Type<Option<usize>>;
pub type TypeDefSealed = TypeDef<usize>;
pub type TypeDefUnsealed = TypeDef<Option<usize>>;

impl TypeDefUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> TypeDefSealed {
        let ty = self.ty.seal(file_version);
        TypeDef {
            comment: self.comment,
            ident: self.ident,
            ty,
            meta: self.meta,
            version: self.version.unwrap_or(file_version),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flatten_simple_oneof() {
        let oneof = Type::OneOf(Box::new(OneOfType {
            types: vec![Type::Builtin(Builtin::I32), Type::Builtin(Builtin::F32)],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        let resolved = oneof.clone().resolve_union();
        assert_eq!(resolved, oneof);
    }

    #[test]
    fn test_flatten_nested_oneof_same_kind() {
        let nested = Type::OneOf(Box::new(OneOfType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::OneOf(Box::new(OneOfType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: OneOfKind::Or,
                    version: 1_usize,
                })),
            ],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        let resolved = nested.resolve_union();
        let expected = Type::OneOf(Box::new(OneOfType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::Builtin(Builtin::F32),
                Type::Builtin(Builtin::Bool),
            ],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_flatten_nested_oneof_different_kind_placeholder() {
        let nested = Type::OneOf(Box::new(OneOfType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::OneOf(Box::new(OneOfType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: OneOfKind::Or,
                    version: 1_usize,
                })),
            ],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        let resolved = nested.resolve_union();
        let expected = Type::OneOf(Box::new(OneOfType {
            types: vec![
                Type::Builtin(Builtin::I32),
                Type::OneOf(Box::new(OneOfType {
                    types: vec![Type::Builtin(Builtin::F32), Type::Builtin(Builtin::Bool)],
                    kind: OneOfKind::Or,
                    version: 1_usize,
                })),
            ],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_flatten_single_type_oneof() {
        let one = Type::OneOf(Box::new(OneOfType {
            types: vec![Type::Builtin(Builtin::I32)],
            kind: OneOfKind::Or,
            version: 1_usize,
        }));
        let resolved = one.resolve_union();
        assert_eq!(resolved, Type::Builtin(Builtin::I32));
    }

    #[test]
    fn test_flatten_non_oneof_type() {
        let ty = TypeUnsealed::Builtin(Builtin::I32);
        let resolved = ty.clone().resolve_union();
        assert_eq!(resolved, ty);
    }
}
