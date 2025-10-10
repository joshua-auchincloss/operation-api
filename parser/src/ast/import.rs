use crate::{
    SpannedToken, Token,
    ast::ty::PathOrIdent,
    tokens::{Parse, Peek, ToTokens, Token},
};

pub struct Use {
    pub kw: SpannedToken![use],
    pub namespace: PathOrIdent,
}

impl Parse for Use {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            kw: stream.parse()?,
            namespace: PathOrIdent::parse(stream)?,
        })
    }
}

impl ToTokens for Use {
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        tt.write(&self.kw);
        tt.space();
        tt.write(&self.namespace);
    }
}

impl Peek for Use {
    fn is(token: &Token) -> bool {
        <Token![use]>::is(token)
    }
}
