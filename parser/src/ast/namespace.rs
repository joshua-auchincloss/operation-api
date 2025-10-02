use crate::{SpannedToken, ast::comment::CommentStream, tokens::Parse};

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
