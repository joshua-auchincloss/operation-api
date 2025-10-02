use crate::{
    SpannedToken, Token,
    tokens::{Parse, Peek, SpannedToken},
};

pub struct OneOfInner {
    kw: SpannedToken![oneof],
    paren: (),
    // variants: Vec<>
}

pub struct OneOf {}

impl Parse for OneOf {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        todo!()
    }
}

impl Peek for OneOf {
    fn is(token: &crate::tokens::tokens::SpannedToken) -> bool {
        <Token![oneof]>::is(token)
    }
}
