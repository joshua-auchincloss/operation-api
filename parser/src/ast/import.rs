use crate::{
    SpannedToken, Token,
    tokens::{Parse, Peek, ToTokens, Token},
};

pub struct Import {
    pub kw: SpannedToken![import],
    pub path: SpannedToken![string],
}

impl Parse for Import {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            kw: stream.parse()?,
            path: stream.parse()?,
        })
    }
}

impl ToTokens for Import {
    fn tokens(&self) -> crate::tokens::MutTokenStream {
        let mut tt = crate::tokens::MutTokenStream::with_capacity(2);

        tt.push(self.kw.token());
        tt.push(self.path.token());

        tt
    }
}

impl Peek for Import {
    fn is(token: &Token) -> bool {
        <Token![import]>::is(token)
    }
}
