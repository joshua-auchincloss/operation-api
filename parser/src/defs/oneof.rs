use crate::{defs::*, parser::Rule};
use pest::iterators::Pairs;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum OneOfKind {
    Or,
}

#[derive(Debug, bon::Builder, Clone, PartialEq)]
pub struct OneOfType<V> {
    pub types: Vec<Type<V>>,
    pub kind: OneOfKind,
    pub version: V,
}

impl<V: Default + Clone + std::fmt::Debug + PartialEq> FromInner for OneOfType<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut types = vec![];
        let mut kind = OneOfKind::Or;
        for pair in pairs {
            match pair.as_rule() {
                Rule::type_operand | Rule::typ => return Self::from_inner(pair.into_inner()),
                Rule::singular_type => types.push(Type::from_inner(pair.into_inner())?),
                Rule::oneof => {
                    kind = OneOfKind::Or;
                    for it in pair.into_inner() {
                        types.push(Type::from_inner(Pairs::single(it))?)
                    }
                },
                rule => panic!("{rule:#?}"),
            }
        }
        Ok(Self {
            types,
            kind,
            version: V::default(),
        })
    }
}

impl OneOfType<usize> {
    pub fn cast_value(
        &self,
        pairs: Pairs<Rule>,
    ) -> crate::Result<Value> {
        for ty in &self.types {
            match ty {
                Type::OneOf(oneof) => {
                    if let Ok(value) = oneof.cast_value(pairs.clone()) {
                        return Ok(value);
                    }
                },
                ty => {
                    if let Ok(value) = Value::from_inner(pairs.clone(), ty.clone()) {
                        return Ok(value);
                    }
                },
            }
        }
        Err(crate::Error::value_error(
            pairs.as_str().into(),
            vec![Type::oneof(self.clone())],
        ))
    }
}

pub type OneOfSealed = OneOfType<usize>;
pub type OneOfUnsealed = OneOfType<Option<usize>>;

impl OneOfUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> OneOfSealed {
        OneOfType {
            types: self
                .types
                .into_iter()
                .map(|it| it.seal(file_version))
                .collect(),
            kind: self.kind,
            version: self.version.unwrap_or(file_version),
        }
    }
}
