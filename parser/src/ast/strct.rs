use crate::{
    ast::{comment::CommentStream, ty::Type},
    tokens::{LexingError, TokenStream},
};

use crate::{
    SpannedToken, Token,
    defs::Spanned,
    tokens::{Brace, ImplDiagnostic, Parse, Peek, Repeated, SpannedToken, brace},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Sep {
    Required {
        sep: SpannedToken![:],
    },
    Optional {
        q: SpannedToken![?],
        sep: SpannedToken![:],
    },
}

impl Parse for Sep {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(if stream.peek::<Token![?]>() {
            Self::Optional {
                q: stream.parse()?,
                sep: stream.parse()?,
            }
        } else {
            Self::Required {
                sep: stream.parse()?,
            }
        })
    }
}

impl ImplDiagnostic for Sep {
    fn fmt() -> &'static str {
        "?: | :"
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Arg {
    pub comments: CommentStream,
    pub name: SpannedToken![ident],
    pub sep: Spanned<Sep>,
    pub typ: Type,
}

impl ImplDiagnostic for Arg {
    fn fmt() -> &'static str {
        "foo?: i32"
    }
}

impl Parse for Arg {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            comments: CommentStream::parse(stream)?,
            name: stream.parse()?,
            sep: stream.parse()?,
            typ: Type::parse(stream)?,
        })
    }
}

impl Peek for Arg {
    fn is(token: &crate::tokens::tokens::SpannedToken) -> bool {
        <Token![ident]>::is(token)
    }
}

pub struct Struct {
    pub kw: SpannedToken![struct],
    pub name: SpannedToken![ident],
    pub brace: Brace,
    pub args: Repeated<Arg, Token![,]>,
}

impl Parse for Struct {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut braced;
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
            brace: brace!(braced in stream),
            args: Repeated::parse(&mut braced)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::tokens::{ToTokens, tokenize};

    use super::*;

    #[test_case::test_case("struct Foo { /* comment before */ bar: i32 }", "Foo", vec![("bar", "i32", "comment before", true)]; "single required arg")]
    #[test_case::test_case("struct Foo { bar?: i32 }", "Foo", vec![("bar", "i32", "", false)]; "single optional arg")]
    #[test_case::test_case("struct Foo { bar: i32, /* other comment */ baz?: String }", "Foo", vec![("bar", "i32", "", true), ("baz", "String", "other comment", false)]; "multiple args")]
    fn test_parse_struct_args(
        src: &str,
        expected_name: &str,
        expected_args: Vec<(&str, &str, &str, bool)>,
    ) {
        let mut stream = tokenize(src).unwrap();
        let parsed = Struct::parse(&mut stream).expect("Should parse struct");

        assert_eq!(parsed.name.borrow_string(), expected_name);

        for (pos, (expect_name, expect_ty, expect_comment, expect_required)) in
            expected_args.into_iter().enumerate()
        {
            let field = parsed.args.values.get(pos).unwrap();

            assert_eq!(field.value.name.borrow_string(), expect_name);
            assert_eq!(&format!("{}", field.value.typ.tokens()), expect_ty);
            assert_eq!(
                &format!("{}", field.value.comments.tokens()),
                expect_comment
            );

            assert!(if expect_required {
                matches!(field.value.sep.value, Sep::Required { .. })
            } else {
                matches!(field.value.sep.value, Sep::Optional { .. })
            })
        }
    }
}
