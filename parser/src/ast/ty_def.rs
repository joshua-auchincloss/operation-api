use crate::{
    SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{Parse, Peek},
};

pub struct NamedType {
    pub kw: SpannedToken![type],
    pub name: SpannedToken![ident],
    pub eq: SpannedToken![=],
    pub ty: Spanned<Type>,
}

impl Parse for NamedType {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
            eq: stream.parse()?,
            ty: stream.parse()?,
        })
    }
}

impl Peek for NamedType {
    fn is(token: &crate::tokens::Token) -> bool {
        <Token![type]>::is(token)
    }
}
