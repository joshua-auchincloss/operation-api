use crate::{
    SpannedToken, Token,
    tokens::{Parse, Peek, ToTokens, Token},
};

pub struct Use {
    pub kw: SpannedToken![use],
    pub namespace: SpannedToken![path],
}

impl Parse for Use {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            kw: stream.parse()?,
            namespace: stream.parse()?,
        })
    }
}

impl ToTokens for Use {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        tt.push(self.kw.token());
        tt.push(self.namespace.token());
    }
}

impl Peek for Use {
    fn is(token: &Token) -> bool {
        <Token![use]>::is(token)
    }
}
