use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Builtin {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    // F16,
    F32,
    F64,
    Bool,
    Str,
    Null,
}

macro_rules! builtin_parse {
    ($self: ident $v: ident, $($t: ident), + $(,)?) => {
        match $self {
            Self::Null => Ok(Value::Null),
            $(
                Self::$t => Ok(Value::$t($v.parse()?)),
            )*

        }
    };
}

impl Builtin {
    pub fn value(&self, v: &str) -> crate::Result<Value> {
        builtin_parse! {
            self v,
            I8,  I16, I32, I64,
            U8,  U16, U32, U64,
            F32, F64,
            Bool,
            Str
        }
    }
}

impl FromInner for Builtin {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        for it in pairs {
            return Ok(match it.as_rule() {
                Rule::builtin => return Self::from_inner(it.into_inner()),
                Rule::ints | Rule::unsigned | Rule::floats => {
                    return Self::from_inner(it.into_inner());
                }
                Rule::i8 => Self::I8,
                Rule::i16 => Self::I16,
                Rule::i32 => Self::I32,
                Rule::i64 => Self::I64,
                Rule::u8 => Self::U8,
                Rule::u16 => Self::U16,
                Rule::u32 => Self::U32,
                Rule::u64 => Self::U64,
                // Rule::f16 => Self::F16,
                Rule::f32 => Self::F32,
                Rule::f64 => Self::F64,
                Rule::bool => Self::Bool,
                Rule::str => Self::Str,
                Rule::null => Self::Null,
                rule => panic!("unknown builtin: {rule:#?}"),
            });
        }
        Err(crate::Error::def::<Self>(Rule::builtin))
    }
}
