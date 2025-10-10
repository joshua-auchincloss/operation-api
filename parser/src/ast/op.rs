use crate::{
    Parse, Peek, SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{Paren, Repeated, ToTokens, Token, paren},
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
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        tt.write(&self.kw);
        tt.space();
        tt.write(&self.name);

        self.paren.write_with(tt, |tt| {
            if let Some(args) = &self.args {
                for (idx, item) in args.value.values.iter().enumerate() {
                    tt.write(&item.value);
                    if idx < args.value.values.len() - 1 {
                        tt.token(&Token::Comma);
                        tt.space();
                    }
                }
            }
        });

        tt.space();
        tt.write(&self.ret);
        tt.space();
        tt.write(&self.return_type);
    }
}
