use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum UnionKind {
    Or,
    And,
}

#[derive(Debug, bon::Builder, Clone, PartialEq)]
pub struct UnionType {
    pub types: Vec<Type>,
    pub kind: UnionKind,
}

impl FromInner for UnionType {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut types = vec![];
        let mut kind = UnionKind::Or;

        for pair in pairs {
            match pair.as_rule() {
                Rule::type_operand | Rule::typ => return Self::from_inner(pair.into_inner()),
                Rule::singular_type => types.push(Type::from_inner(pair.into_inner())?),
                Rule::oneof => {
                    kind = UnionKind::Or;
                    for it in pair.into_inner() {
                        types.push(Type::from_inner(Pairs::single(it))?)
                    }
                },
                rule => panic!("{rule:#?}"),
            }
        }

        Ok(Self { types, kind })
    }
}

impl UnionType {
    pub fn cast_value(
        &self,
        pairs: Pairs<Rule>,
    ) -> crate::Result<Value> {
        for ty in &self.types {
            match ty {
                Type::Union(union) => {
                    if let Ok(value) = union.cast_value(pairs.clone()) {
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
            vec![Type::union(self.clone())],
        ))
    }
}
