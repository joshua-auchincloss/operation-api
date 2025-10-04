use crate::{
    ast::{
        comment::{CommentAst, CommentStream},
        ty::Type,
    },
    tokens::{self, Token},
};

use crate::{
    SpannedToken, Token,
    defs::Spanned,
    tokens::{Brace, ImplDiagnostic, Parse, Peek, Repeated, brace},
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

impl tokens::ToTokens for Sep {
    fn tokens(&self) -> tokens::MutTokenStream {
        let mut tt = tokens::MutTokenStream::new();

        if matches!(self, Self::Optional { .. }) {
            tt.push(<Token![?]>::new().token());
        }

        tt.push(<Token![:]>::new().token());

        tt
    }
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

impl tokens::ToTokens for Arg {
    fn tokens(&self) -> tokens::MutTokenStream {
        let mut tt = tokens::MutTokenStream::new();
        self.comments.write(&mut tt);
        self.name.write(&mut tt);
        self.sep.write(&mut tt);
        self.typ.write(&mut tt);
        tt
    }
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
    fn is(token: &crate::tokens::tokens::Token) -> bool {
        <Token![ident]>::is(token)
    }
    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        let mut fork = stream.fork();
        while fork.peek::<CommentAst>() {
            if fork.parse::<Spanned<CommentAst>>().is_err() {
                break;
            }
        }
        fork.peek::<Token![ident]>()
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

impl tokens::Peek for Struct {
    fn is(token: &Token) -> bool {
        <Token![struct]>::is(token)
    }
}

impl tokens::ToTokens for Struct {
    fn tokens(&self) -> tokens::MutTokenStream {
        let mut tt = tokens::MutTokenStream::new();
        self.kw.write(&mut tt);
        self.name.write(&mut tt);
        tt.push(tokens::LBraceToken::new().token());
        self.args.write(&mut tt);
        tt.push(tokens::RBraceToken::new().token());
        tt
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
                &format!(
                    "{}",
                    field
                        .value
                        .value
                        .comments
                        .comments()
                        .map(Clone::clone)
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
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
