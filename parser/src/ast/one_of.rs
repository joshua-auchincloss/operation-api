use crate::{
    SpannedToken, Token,
    ast::{comment::CommentStream, ty::Type},
    defs::Spanned,
    tokens::{
        Brace, ImplDiagnostic, MutTokenStream, Paren, Parse, Peek, Repeated, ToTokens, brace,
        paren, tokens,
    },
};

// something like: oneof a | b | i32[] | (str | bool)[][]
// We store variants as a Repeated list separated by '|', preserving separator spans.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AnonymousOneOf {
    pub(crate) kw: SpannedToken![oneof],
    pub(crate) variants: Spanned<Repeated<Type, Token![|]>>,
}

impl ToTokens for AnonymousOneOf {
    fn tokens(&self) -> crate::tokens::MutTokenStream {
        let mut tt = MutTokenStream::new();
        tt.push(self.kw.token());
        for item in &self.variants.value.values {
            item.value.write(&mut tt);
            if let Some(sep) = &item.sep {
                sep.write(&mut tt);
            }
        }
        tt
    }
}

impl ImplDiagnostic for AnonymousOneOf {
    fn fmt() -> &'static str {
        "oneof abc | def | i32"
    }
}

impl Parse for AnonymousOneOf {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        tracing::trace!(cursor=%stream.cursor(), "parsing oneof");
        let kw: SpannedToken![oneof] = stream.parse()?;

        let first: Spanned<Type> = stream.parse()?;
        let mut values = Vec::new();
        let mut sep: Option<Spanned<Token![|]>> = None;

        if stream.peek::<Token![|]>() {
            sep = Some(stream.parse()?);
        }

        values.push(crate::tokens::ast::RepeatedItem {
            value: first,
            sep: sep.clone(),
        });

        while let Some(..) = sep {
            if !stream.peek::<Type>() {
                break;
            }
            let next: Spanned<Type> = stream.parse()?;
            let mut next_sep: Option<Spanned<Token![|]>> = None;
            if stream.peek::<Token![|]>() {
                next_sep = Some(stream.parse()?);
            }
            values.push(crate::tokens::ast::RepeatedItem {
                value: next,
                sep: next_sep.clone(),
            });
            sep = next_sep;
        }

        let end_span = values
            .last()
            .map(|v| v.value.span.clone())
            .unwrap();

        let variants = Spanned::new(
            end_span.start,
            end_span.end,
            crate::tokens::ast::Repeated { values },
        );
        Ok(Self { kw, variants })
    }
}

impl Peek for AnonymousOneOf {
    fn is(token: &tokens::Token) -> bool {
        <Token![oneof]>::is(token)
    }
}

// we either have `a(i32)` or `b { desc: str }`
#[derive(serde::Deserialize, serde::Serialize)]
pub enum OneOfVariants {
    Tuple {
        comments: CommentStream,
        name: SpannedToken![ident],
        paren: Paren,
        inner: Type,
    },
    LocalStruct {
        comments: CommentStream,
        name: SpannedToken![ident],
        brace: Brace,
        fields: Spanned<Repeated<super::strct::Arg, Token![,]>>,
    },
}

impl Parse for OneOfVariants {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        tracing::trace!(cursor=%stream.cursor(), "parsing oneof variant");
        let comments = CommentStream::parse(stream)?;
        let name = stream.parse()?;

        let mut inner;

        Ok(if stream.peek::<tokens::LBraceToken>() {
            tracing::trace!("parsing local struct variant");
            let brace = brace!(inner in stream);
            let fields = inner.parse()?;
            Self::LocalStruct {
                comments,
                name,
                brace,
                fields,
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
pub struct OneOf {}

impl Parse for OneOf {
    #[allow(unused)]
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        todo!()
    }
}

impl Peek for OneOf {
    fn is(token: &crate::tokens::tokens::Token) -> bool {
        <Token![oneof]>::is(token)
    }
}
