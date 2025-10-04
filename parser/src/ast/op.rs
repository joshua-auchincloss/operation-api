use crate::{
    Parse, Peek, SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{self, Paren, Repeated, ToTokens, paren},
};

pub struct Operation {
    pub kw: SpannedToken![operation],
    pub name: SpannedToken![ident],
    pub paren: Paren,
    pub args: Option<Spanned<Repeated<super::strct::Arg, Token![,]>>>,
    pub ret: SpannedToken![->],
    pub return_type: Spanned<Type>,
}

impl Parse for Operation {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut args;
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
            paren: paren!(args in stream),
            args: Option::parse(&mut args)?,
            ret: stream.parse()?,
            return_type: stream.parse()?,
        })
    }
}

impl Peek for Operation {
    fn is(token: &crate::tokens::Token) -> bool {
        <Token![operation]>::is(token)
    }
}

impl ToTokens for Operation {
    fn tokens(&self) -> crate::tokens::MutTokenStream {
        let mut tt = crate::tokens::MutTokenStream::new();

        tt.write(&self.kw);
        tt.write(&self.name);
        tt.write(&tokens::LParenToken::new());
        tt.write(&self.args);
        tt.write(&tokens::RParenToken::new());
        tt.write(&self.ret);
        tt.write(&self.return_type);

        tt
    }
}
