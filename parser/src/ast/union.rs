use crate::{
    SpannedToken,
    ast::ty::PathOrIdent,
    bail_unchecked,
    defs::Spanned,
    tokens::{self, ImplDiagnostic, Paren, Parse, Peek, Repeated, ToTokens, paren},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum IdentOrUnion {
    Ident(PathOrIdent),
    Union {
        paren: Paren,
        inner: Spanned<Box<Union>>,
    },
}

impl ToTokens for IdentOrUnion {
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        match self {
            Self::Ident(iden) => iden.write(tt),
            Self::Union { inner, paren } => paren.write_with(tt, |tt| tt.write(inner)),
        }
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
                (stream.cursor(), stream.cursor())
            };
            return Ok(IdentOrUnion::Union {
                paren,
                inner: Spanned::new(start, end, Box::new(inner_union)),
            });
        }
        if stream.peek::<tokens::IdentToken>() {
            return Ok(IdentOrUnion::Ident(PathOrIdent::parse(stream)?));
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

#[derive(serde::Serialize, serde::Deserialize)]
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
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        for (i, item) in self.types.values.iter().enumerate() {
            if i > 0 {
                tt.space();
            }
            item.value.write(tt);
            if let Some(sep) = &item.sep {
                tt.space();
                sep.write(tt);
            }
        }
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
