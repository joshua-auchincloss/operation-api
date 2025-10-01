use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone)]
pub enum Value {
    // String(String),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    // F16(f16),
    F32(f32),
    F64(f64),
    Bool(bool),
    Str(String),
    Null,
    Ident(Ident),
}

impl PartialEq for Value {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Value::Ident(a), Value::Ident(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::I8(a), Value::I8(b)) => a == b,
            (Value::I16(a), Value::I16(b)) => a == b,
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::U8(a), Value::U8(b)) => a == b,
            (Value::U16(a), Value::U16(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn from_inner(
        pairs: Pairs<crate::parser::Rule>,
        ty: Type,
    ) -> crate::Result<Self> {
        use crate::parser::Rule;
        for pair in pairs {
            match pair.as_rule() {
                Rule::value | Rule::number | Rule::quoted_value => {
                    // Try to coerce value to the correct type
                    match &ty {
                        Type::Builtin(builtin) => {
                            match builtin {
                                Builtin::I8
                                | Builtin::I16
                                | Builtin::I32
                                | Builtin::I64
                                | Builtin::U8
                                | Builtin::U16
                                | Builtin::U32
                                | Builtin::U64
                                | Builtin::F32
                                | Builtin::F64
                                | Builtin::Bool
                                | Builtin::Str => {
                                    return builtin.value(pair.as_str());
                                },
                                Builtin::Null => {
                                    return Ok(Value::Null);
                                },
                            }
                        },
                        Type::Union(union) => {
                            return union.cast_value(Pairs::single(pair)); // try to coerce to one of the union types
                        },
                        Type::Ident(_) => {
                            // for user-defined types, treat as ident
                            return Ok(Value::Ident(Ident::from(pair.as_str())));
                        },
                        Type::Enum(_) => {
                            // for enums, treat as ident
                            return Ok(Value::Ident(Ident::from(pair.as_str())));
                        },
                        Type::Array(_) => todo!(),
                    }
                },
                Rule::eq_value => {
                    return Value::from_inner(pair.into_inner(), ty);
                },
                Rule::quoted => {
                    // only for str
                    return Self::from_inner(pair.into_inner(), ty);
                },
                Rule::ident => return Ok(Value::Ident(Ident::from(pair.as_str()))),
                _ => {},
            }
        }
        Err(crate::Error::def::<Self>(Rule::value))
    }
}
