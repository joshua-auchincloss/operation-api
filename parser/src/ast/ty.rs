use crate::{
    SpannedToken, Token,
    ast::{array::Array, def::EnumDef, one_of::AnonymousOneOf},
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

            impl Peek for Builtin {
                fn is(token: &tokens::SpannedToken) -> bool {
                    false  $(
                       || crate::tokens::tokens::[<Kw $t Token>]::is(token)
                    )*
                }
            }

            impl ImplDiagnostic for Builtin {
                fn fmt() -> &'static str {
                    "builtin (i16, i32, str, ...)"
                }
            }

            impl ToTokens for Builtin {
                fn tokens(&self) -> MutTokenStream {
                    let mut tt = MutTokenStream::with_capacity(1);
                    tt.push(match self {
                        $(
                            Self::$t(t) => t.value.token(),
                        )*
                    });
                    tt
                }
            }


            impl Parse for Builtin {
                fn parse(stream: &mut TokenStream) -> AstResult<Self> {
                    $(
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Builtin { ty: Spanned<Builtin> },
    Ident { to: SpannedToken![ident] },
    OneOf { ty: Spanned<AnonymousOneOf> },
    Array { ty: Spanned<Array> },
    Paren { paren: Paren, ty: Spanned<Box<Type>>,  },
}

impl Parse for Type {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        let mut current: Spanned<Type> = if stream.peek::<AnonymousOneOf>() {
            let one: Spanned<AnonymousOneOf> = stream.parse()?;
            Spanned::new(one.span.start, one.span.end, Type::OneOf { ty: one })
        } else if stream.peek::<tokens::LParenToken>() {
            let mut inner_tokens;
            let paren = paren!(inner_tokens in stream);
            let ty: Spanned<Box<Type>> = inner_tokens.parse()?;

            Spanned::new(paren.span().start, paren.span().end, Type::Paren { paren, ty })
        } else if stream.peek::<Builtin>() {
            let bi: Spanned<Builtin> = stream.parse()?;

            Spanned::new(bi.span.start, bi.span.end, Type::Builtin { ty: bi })
        } else if stream.peek::<Token![ident]>() {
            let ident: SpannedToken![ident] = stream.parse()?;
            Spanned::new(ident.span.start, ident.span.end, Type::Ident { to: ident })
        } else {
            let expect = vec![
                AnonymousOneOf::fmt(),
                Builtin::fmt(),
                <Token![ident]>::fmt(),
            ];
            return Err(if let Some(next) = stream.peek_unchecked() {
                LexingError::expected_oneof(expect, next.value.clone())
            } else {
                LexingError::empty_oneof(expect)
            });
        };


        // we need to be careful here and manually parse the Array types. this is because if
        // we use Array::parse, we can run into infinite recursion errors as the array parses the inner types
        while stream.peek::<tokens::LBracketToken>() {
            let mut inner_tokens;
            let bracket = bracket!(inner_tokens in stream); // unit ()
            let size: Option<SpannedToken![number]> = if inner_tokens.peek::<Token![number]>() {
                Some(inner_tokens.parse()?)
            } else {
                None
            };

            let start = current.span.start;
            let end = stream
                .tokens
                .get(stream.cursor - 1)
                .expect("cursor after consuming RBracket")
                .span
                .end;

            // move current into boxed inner type
            let inner_spanned = current; // move
            let array_value = match size {
                Some(sz) => {
                    Array::Sized {
                        ty: Box::new(inner_spanned),
                        bracket,
                        size: sz,
                    }
                },
                None => {
                    Array::Unsized {
                        ty: Box::new(inner_spanned),
                        bracket,
                    }
                },
            };
            let array_spanned = Spanned::new(start, end, array_value);
            current = Spanned::new(start, end, Type::Array { ty: array_spanned });
        }

        Ok(current.value)
    }
}

impl Peek for Type {
    fn peek(stream: &TokenStream) -> bool {
        stream.peek::<AnonymousOneOf>()
            || stream.peek::<Builtin>()
            || stream.peek::<Token![ident]>()
            || stream.peek::<tokens::LParenToken>()
    }
}

impl ToTokens for Type {
    fn tokens(&self) -> MutTokenStream {
        match self {
            Self::Builtin { ty } => ty.tokens(),
            Self::Ident { to } => to.tokens(),
            Self::OneOf { ty } => ty.tokens(),
            Self::Array { ty } => ty.tokens(),
            Self::Paren { ty, .. } => {
                let mut tt = MutTokenStream::new();
                tt.push(Token::LParen);
                ty.write(&mut tt);
                tt.push(Token::RParen);
                tt
            },
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{
        defs::Spanned,
        tokens::{ToTokens, tokenize},
    };

    #[test_case::test_case(
        "i32", 
        serde_json::json!({"span":{"end":3,"start":0},"value":{"builtin":{"ty":{"span":{"end":3,"start":0},"value":{"span":{"end":3,"start":0},"type":"i32","value":null}}}}}); 
        "parses i32"
    )]
    // simplified: ensure we parse and round trip but we don't assert full JSON for oneof here
    #[test_case::test_case(
        "oneof i32 | i64 | str", 
        serde_json::json!({}); 
        "parses oneof with builtins"
    )]
    #[test_case::test_case(
        "i32[]",
        serde_json::json!({"span":{"end":5,"start":0},"value":{"array":{"ty":{"span":{"end":5,"start":0},"value":{"unsized":{"bracket":null,"ty":{"span":{"end":3,"start":0},"value":{"builtin":{"ty":{"span":{"end":3,"start":0},"value":{"span":{"end":3,"start":0},"type":"i32","value":null}}}}}}}}}}});
        "parses unsized single level array with builtin"
    )]
    #[test_case::test_case(
        "i32[][][]", 
        serde_json::json!({});
        "parses unsized multi level array with builtin"
    )]
    fn test_stp(
        src: &str,
        expect: serde_json::Value,
    ) {
        let mut tt = tokenize(src).unwrap();
        let p: Spanned<super::Type> = tt.parse().unwrap();

        let as_j = serde_json::to_value(&p).unwrap();

        let found = serde_json::to_string(&as_j).unwrap();
        println!("found {found}");

        assert_eq!(
            as_j, expect,
            "expected spans must equal returned spans for builtin types"
        );
    }

    #[test_case::test_case("i8")]
    #[test_case::test_case("i16")]
    #[test_case::test_case("i32")]
    #[test_case::test_case("i64")]
    #[test_case::test_case("u8")]
    #[test_case::test_case("u16")]
    #[test_case::test_case("u32")]
    #[test_case::test_case("u64")]
    #[test_case::test_case("f16")]
    #[test_case::test_case("f32")]
    #[test_case::test_case("f64")]
    #[test_case::test_case("bool")]
    #[test_case::test_case("str")]
    #[test_case::test_case("i32 []"; "round trip unsized array")]
    #[test_case::test_case("i64 [] []"; "round trip double unsized array")]
    #[test_case::test_case("i32 [10]"; "round trip sized array")]
    #[test_case::test_case("i64 [] [5]"; "round trip mixed sized/unsized array")]
    #[test_case::test_case("oneof i32 | i64"; "round trip oneof builtins")]
    #[test_case::test_case("oneof i32 [] | i64 [] []"; "round trip oneof arrays")]
    #[test_case::test_case("oneof i32 | i64 | str | bool | f32 | u8 []"; "round trip nested oneof")]
    #[test_case::test_case("str [] [] []"; "round trip triple unsized array")]
    #[test_case::test_case("bool [42]"; "round trip sized bool array")]
    fn round_trip(src: &str) {
        let mut tt = tokenize(src).unwrap();
        let p: Spanned<super::Type> = tt.parse().unwrap();

        let tokens = p.tokens();

        println!("{tokens:#?}");
        
        let out = format!("{tokens}");

        assert_eq!(out, src);
    }
}
