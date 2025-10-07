use crate::{
    SpannedToken, Token,
    ast::{anonymous::AnonymousStruct, comment::CommentStream, ty::Type},
    defs::Spanned,
    tokens::{ImplDiagnostic, LParenToken, Paren, Parse, Peek, RParenToken, ToTokens, paren, toks},
};

// we either have `a(i32)` or `b { desc: str }`
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Variant {
    Tuple {
        comments: CommentStream,
        name: SpannedToken![ident],
        paren: Paren,
        inner: Type,
    },
    LocalStruct {
        comments: CommentStream,
        name: SpannedToken![ident],
        inner: Spanned<AnonymousStruct>,
    },
}

impl ImplDiagnostic for Variant {
    fn fmt() -> &'static str {
        "a(i32) | b { desc: str }"
    }
}

impl Parse for Variant {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        tracing::trace!(cursor=%stream.cursor(), "parsing oneof variant");
        let comments = CommentStream::parse(stream)?;
        let name = stream.parse()?;

        let mut inner;

        Ok(if stream.peek::<toks::LBraceToken>() {
            tracing::trace!("parsing local struct variant");
            Self::LocalStruct {
                comments,
                name,
                inner: stream.parse()?,
            }
        } else {
            tracing::trace!("parsing tuple variant");
            let paren = paren!(inner in stream);
            let inner = Type::parse(&mut inner)?;
            Self::Tuple {
                comments,
                name,
                paren,
                inner,
            }
        })
    }
}

impl Peek for Variant {
    fn is(token: &toks::Token) -> bool {
        <Token![ident]>::is(token)
    }
}

impl ToTokens for Variant {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        match self {
            Self::LocalStruct {
                comments,
                name,
                inner,
                ..
            } => {
                tt.write(comments);
                tt.write(name);
                tt.write(inner);
            },
            Self::Tuple {
                comments,
                name,
                inner,
                ..
            } => {
                tt.write(comments);
                tt.write(name);
                tt.write(&LParenToken::new());
                tt.write(inner);
                tt.write(&RParenToken::new());
            },
        }
    }
}

macro_rules! variadic {
    ($name: ident: [$kw: ty]) => {
        pub struct $name {
            pub kw: $kw,
            pub name: crate::SpannedToken![ident],
            pub brace: crate::tokens::Brace,
            pub variants: crate::tokens::Repeated<crate::ast::variadic::Variant, crate::Token![,]>,
        }


        impl crate::tokens::Parse for $name {
            fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
                let mut inner;
                Ok(Self {
                    kw: stream.parse()?,
                    name: stream.parse()?,
                    brace: crate::tokens::brace!(inner in stream),
                    variants: crate::tokens::Repeated::parse(&mut inner)?,
                })
            }
        }

        impl crate::tokens::Peek for $name {
            fn is(token: &crate::tokens::toks::Token) -> bool {
                <$kw>::is(token)
            }
        }

        impl crate::tokens::ToTokens for $name {
            fn write(&self, tt: &mut crate::tokens::MutTokenStream) {
                tt.write(&self.kw);
                tt.write(&self.name);
                tt.write(&crate::tokens::LBraceToken::new());
                tt.write(&self.variants);
                tt.write(&crate::tokens::RBraceToken::new());
            }
        }

    };
}

pub(crate) use variadic;

#[cfg(test)]
mod test {
    #[test_case::test_case("a (i32)"; "type variant")]
    #[test_case::test_case("b {desc: str}"; "type anonymous struct")]
    #[test_case::test_case("/* some comment */ b {desc: str}"; "type anonymous struct with comment before")]
    #[test_case::test_case("b {// some comment\ndesc: str}"; "type anonymous struct with sl comment in fields")]
    #[test_case::test_case("b {/* some\ncomment */ desc: str}"; "type anonymous struct with ml comment in fields")]
    fn round_trip(src: &str) {
        crate::tst::round_trip::<super::Variant>(src).unwrap();
    }
}
