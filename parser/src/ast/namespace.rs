use crate::{
    SpannedToken, Token,
    tokens::{Parse, Peek, Token},
};

pub struct Namespace {
    pub kw: SpannedToken![namespace],
    pub name: SpannedToken![ident],
}

impl Parse for Namespace {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
        })
    }
}

impl Peek for Namespace {
    fn is(token: &Token) -> bool {
        <Token![namespace]>::is(token)
    }
}
