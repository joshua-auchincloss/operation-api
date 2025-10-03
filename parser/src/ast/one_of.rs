use crate::{
    SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{ImplDiagnostic, MutTokenStream, Parse, Peek, Repeated, ToTokens, tokens},
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

super::variadic::variadic! {
    OneOf: [SpannedToken![oneof]]
}
