use crate::{
    SpannedToken,
    ast::{def::EnumDef, one_of::OneOfInner},
    defs::Spanned,
    tokens::*,
};

macro_rules! builtin {
    ($($t: ident), + $(,)?) => {
        paste::paste!{
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(tag = "type", rename_all = "snake_case")]
            pub enum Builtin {
                $(
                    $t(crate::defs::Spanned<crate::tokens::tokens::[<Kw $t Token>]>),
                )*
            }
            impl Parse for Builtin {
                fn parse(stream: &mut TokenStream) -> AstResult<Self> {
                    if false {} $(
                        if stream.peek::<crate::tokens::tokens::[<Kw $t Token>]>() {
                            return Ok(Self::$t(
                                stream.parse()?
                            ))
                        }
                    )*

                    let tys: Vec<_> = vec![
                        $(
                            crate::tokens::tokens::[<Kw $t Token>]::fmt(),
                        )*
                    ];

                    let next = stream.next().ok_or_else(
                        || LexingError::empty_oneof(tys.clone())
                    )?;
                    Err(
                        LexingError::expected_oneof(
                            tys.clone(), next.value
                        )
                    )
                }
            }
        }


    };
}

builtin! {
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F16,
    F32,
    F64,

    Bool,
    Str,
}

#[derive()]
pub enum Type {
    Builtin(Spanned<Builtin>),
    Ident(SpannedToken![ident]),
    OneOf(Spanned<OneOfInner>),
    Enum(EnumDef),
    Array(Box<Type>),
    SizedArray { ty: Box<Type>, size: usize },
}

// impl<V> Type<V> {
//     pub fn builtin(value: Builtin) -> Self {
//         Self::Builtin(value)
//     }

//     pub fn ident<I: Into<Ident>>(value: I) -> Self {
//         Self::Ident(value.into())
//     }

//     pub fn oneof(value: OneOfType<V>) -> Self {
//         Self::OneOf(Box::new(value))
//     }

//     pub fn array(value: Type<V>) -> Self {
//         Self::Array(Box::new(value))
//     }
// }

#[cfg(test)]
mod test {
    use crate::{ast::ty::Builtin, defs::Spanned, tokens::tokenize};

    #[test_case::test_case(
        "i32", serde_json::json!({"span":{"end":1,"start":0},"value":{"span":{"end":1,"start":0},"type":"i32","value":null}}); "parses i32"
    )]
    fn test_stp(
        src: &str,
        expect: serde_json::Value,
    ) {
        let mut tt = tokenize(src).unwrap();
        let p: Spanned<Builtin> = tt.parse().unwrap();

        let as_j = serde_json::to_value(&p).unwrap();

        let found = serde_json::to_string(&as_j).unwrap();
        println!("found {found}");

        assert_eq!(
            as_j, expect,
            "expected spans must equal returned spans for builtin types"
        )
    }
}
