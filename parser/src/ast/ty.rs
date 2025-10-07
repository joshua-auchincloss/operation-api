use crate::{
    SpannedToken, Token,
    ast::{array::Array, one_of::AnonymousOneOf, union::Union},
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
                    $t(crate::defs::Spanned<crate::tokens::toks::[<Kw $t Token>]>),
                )*
            }

            impl Peek for Builtin {
                fn is(token: &toks::Token) -> bool {
                    false  $(
                       || crate::tokens::toks::[<Kw $t Token>]::is(token)
                    )*
                }
            }

            impl ImplDiagnostic for Builtin {
                fn fmt() -> &'static str {
                    "builtin (i16, i32, str, ...)"
                }
            }

            impl ToTokens for Builtin {
                fn write(&self, tt: &mut MutTokenStream) {
                    tt.push(match self {
                        $(
                            Self::$t(t) => t.value.token(),
                        )*
                    });
                }
            }


            impl Parse for Builtin {
                fn parse(stream: &mut TokenStream) -> AstResult<Self> {
                    $(
                        if stream.peek::<crate::tokens::toks::[<Kw $t Token>]>() {
                            return Ok(Self::$t(
                                stream.parse()?
                            ))
                        }
                    )*

                    let tys: Vec<_> = vec![
                        $(
                            crate::tokens::toks::[<Kw $t Token>]::fmt(),
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

    Usize,

    F16,
    F32,
    F64,

    Bool,
    Str,

    DateTime,
    Complex,
    Binary,

    Never
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Builtin {
        ty: Spanned<Builtin>,
    },
    Ident {
        to: SpannedToken![path],
    },
    OneOf {
        ty: Spanned<AnonymousOneOf>,
    },
    Array {
        ty: Spanned<Array>,
    },
    Paren {
        paren: Paren,
        ty: Spanned<Box<Type>>,
    },
    Union {
        ty: Spanned<Union>,
    },
    Result {
        ty: Spanned<Box<Type>>,
        ex: SpannedToken![!],
    },
}

impl Parse for Type {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        tracing::trace!(cursor=%stream.cursor(), "parsing type");
        let start = stream.current_span().start;
        let current: Type = if stream.peek::<AnonymousOneOf>() {
            tracing::trace!("parsing oneof in type");
            Type::OneOf {
                ty: stream.parse()?,
            }
        } else if stream.peek::<Union>() {
            tracing::trace!("parsing union in type");
            Type::Union {
                ty: stream.parse()?,
            }
        } else if stream.peek::<toks::LParenToken>() {
            tracing::trace!("parsing paren type");
            let mut inner;
            let paren = paren!(inner in stream);
            Type::Paren {
                paren,
                ty: inner.parse()?,
            }
        } else if stream.peek::<Builtin>() {
            tracing::trace!("parsing builtin in type");
            Type::Builtin {
                ty: stream.parse()?,
            }
        } else if stream.peek::<Token![ident]>() {
            tracing::trace!("parsing ident in type");
            Type::Ident {
                to: stream.parse()?,
            }
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

        let end = stream.current_span().end;
        let mut current = Spanned::new(start, end, current);

        // we need to be careful here and manually parse the Array types. this is because if
        // we use Array::parse, we can run into infinite recursion errors as the array parses the inner types
        while stream.peek::<toks::LBracketToken>() {
            tracing::trace!("parsing trailing array suffix");
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

        if stream.peek::<Token![!]>() {
            let ex = stream.parse()?;
            current = Spanned::new(
                start,
                ex.span.end,
                Type::Result {
                    ty: current.map(Box::new),
                    ex,
                },
            )
        }
        Ok(current.value)
    }
}

impl Peek for Type {
    fn peek(stream: &TokenStream) -> bool {
        stream.peek::<AnonymousOneOf>()
            || stream.peek::<Builtin>()
            || stream.peek::<Token![ident]>()
            || stream.peek::<toks::LParenToken>()
    }
}

impl ToTokens for Type {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        match self {
            Self::Builtin { ty } => ty.write(tt),
            Self::Ident { to } => to.write(tt),
            Self::OneOf { ty } => ty.write(tt),
            Self::Array { ty } => ty.write(tt),
            Self::Union { ty } => ty.write(tt),
            Self::Result { ty, ex } => {
                ty.write(tt);
                ex.write(tt);
            },
            Self::Paren { ty, .. } => {
                tt.push(Token::LParen);
                ty.write(tt);
                tt.push(Token::RParen);
            },
        }
    }
}

#[cfg(test)]
mod test {

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
    #[test_case::test_case("str!"; "builtin result")]
    #[test_case::test_case("i32 []"; "round trip unsized array")]
    #[test_case::test_case("i64 [] []"; "round trip double unsized array")]
    #[test_case::test_case("i32 [10]"; "round trip sized array")]
    #[test_case::test_case("i64 [] [5]"; "round trip mixed sized/unsized array")]
    #[test_case::test_case("oneof i32 | i64"; "round trip oneof builtins")]
    #[test_case::test_case("oneof i32 [] | i64 [] []"; "round trip oneof arrays")]
    #[test_case::test_case("oneof i32 | i64 | str | bool | f32 | u8 []"; "round trip nested oneof")]
    #[test_case::test_case("str [] [] []"; "round trip triple unsized array")]
    #[test_case::test_case("bool [42]"; "round trip sized bool array")]
    #[test_case::test_case("binary"; "round trip binary")]
    #[test_case::test_case("datetime"; "round trip datetime")]
    #[test_case::test_case("never"; "round trip never")]
    #[test_case::test_case("(oneof i32 | f32) []"; "round trip nested oneof array with paren")]
    #[test_case::test_case("(oneof i32 | f32) [] !"; "round trip nested oneof array with paren result")]
    #[test_case::test_case("oneof my_struct | never"; "round trip ident and never")]
    #[test_case::test_case("my_struct & other_struct"; "basic union")]
    #[test_case::test_case("my_struct & ((other_struct & inner_struct) & next_struct)"; "nested union")]
    fn round_trip(src: &str) {
        crate::tst::round_trip::<super::Type>(src).unwrap();
    }
}
