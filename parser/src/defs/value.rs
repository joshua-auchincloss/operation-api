use crate::defs::*;

use pest::iterators::Pairs;

use crate::parser::Rule;

#[derive(Debug, Clone)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
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
    pub fn from_inner<V: Clone + std::fmt::Debug + PartialEq>(
        pairs: Pairs<crate::parser::Rule>,
        ty: Type<V>,
    ) -> crate::Result<Self> {
        use crate::parser::Rule;
        for pair in pairs {
            match pair.as_rule() {
                Rule::value | Rule::number | Rule::quoted_value => {
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
                        Type::OneOf(_union) => {
                            todo!();
                        },
                        Type::Ident(_) => {
                            return Ok(Value::Ident(Ident::from(pair.as_str())));
                        },
                        Type::Enum(_) => {
                            return Ok(Value::Ident(Ident::from(pair.as_str())));
                        },
                        Type::Array(_) => todo!(),
                    }
                },
                Rule::eq_value => {
                    return Value::from_inner(pair.into_inner(), ty);
                },
                Rule::quoted => {
                    return Self::from_inner(pair.into_inner(), ty);
                },
                Rule::ident => return Ok(Value::Ident(Ident::from(pair.as_str()))),
                _ => {},
            }
        }
        Err(crate::Error::def::<Self>(Rule::value))
    }
}

impl FromPairSpan for Value {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        Err(crate::Error::def::<Self>(Rule::value).with_span(start, end))
    }
}
