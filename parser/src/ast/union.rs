use crate::{
    SpannedToken, bail_unchecked,
    defs::Spanned,
    tokens::{
        self, ImplDiagnostic, LParenToken, Paren, Parse, Peek, RParenToken, Repeated, ToTokens,
        paren,
    },
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IdentOrUnion {
    Ident(SpannedToken![ident]),
    Union {
        paren: Paren,
        inner: Spanned<Box<Union>>,
    },
}

impl ToTokens for IdentOrUnion {
    fn tokens(&self) -> tokens::MutTokenStream {
        let mut tt = tokens::MutTokenStream::new();
        match self {
            Self::Ident(iden) => iden.write(&mut tt),
            Self::Union { inner, .. } => {
                LParenToken::new().write(&mut tt);
                inner.write(&mut tt);
                RParenToken::new().write(&mut tt);
            },
        }
        tt
    }
}

impl ImplDiagnostic for IdentOrUnion {
    fn fmt() -> &'static str {
        "identifier or parenthesized union"
    }
}

impl Peek for IdentOrUnion {
    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        stream.peek::<tokens::IdentToken>() || stream.peek::<tokens::LParenToken>()
    }
}

impl Parse for IdentOrUnion {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        use crate::tokens::error::LexingError;
        if stream.peek::<tokens::LParenToken>() {
            let mut inner;
            let paren = paren!(inner in stream);
            let inner_union: Union = Union::parse(&mut inner)?;
            let (start, end) = if let (Some(first), Some(last)) = (
                inner_union.types.values.first(),
                inner_union.types.values.last(),
            ) {
                (first.value.span.start, last.value.span.end)
            } else {
                // guard anyway
                (stream.cursor(), stream.cursor())
            };
            return Ok(IdentOrUnion::Union {
                paren,
                inner: Spanned::new(start, end, Box::new(inner_union)),
            });
        }
        if stream.peek::<tokens::IdentToken>() {
            return Ok(IdentOrUnion::Ident(stream.parse()?));
        }
        Err(if let Some(next) = stream.peek_unchecked() {
            LexingError::expected_oneof(
                vec![
                    <tokens::IdentToken as ImplDiagnostic>::fmt(),
                    tokens::LParenToken::fmt(),
                ],
                next.value.clone(),
            )
        } else {
            LexingError::empty_oneof(vec![<tokens::IdentToken as ImplDiagnostic>::fmt(), "("])
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Union {
    pub types: Repeated<IdentOrUnion, tokens::AmpToken>,
}

impl ImplDiagnostic for Union {
    fn fmt() -> &'static str {
        "a & b"
    }
}

impl Parse for Union {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            types: Repeated::parse(stream)?,
        })
    }
}

impl Peek for Union {
    fn peek(stream: &tokens::TokenStream) -> bool {
        let mut fork = stream.fork();
        let _: IdentOrUnion = bail_unchecked!(IdentOrUnion::parse(&mut fork); false);
        let _: SpannedToken![&] = bail_unchecked!(fork.parse(); false);
        true
    }
}

impl ToTokens for Union {
    fn tokens(&self) -> tokens::MutTokenStream {
        self.types.tokens()
    }
}

#[cfg(test)]
mod test {
    use super::Union;

    #[test_case::test_case("some_struct & (other_struct & last_struct)"; "inner parenthesized")]
    #[test_case::test_case("some_struct & other_struct & last_struct"; "no paren triplets")]
    fn round_trip(src: &str) {
        crate::tst::round_trip::<Union>(src).unwrap();
    }
}
