use crate::{
    SpannedToken, Token,
    tokens::{Parse, Peek, Repeated, SpannedToken},
};

pub enum Sep {
    Required {
        sep: SpannedToken![:],
    },
    Optional {
        sep: SpannedToken![:],
        q: SpannedToken![?],
    },
}

impl Parse for Sep {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let sep = stream.parse()?;
        Ok(if stream.peek::<Token![?]>() {
            Self::Optional {
                sep,
                q: stream.parse()?,
            }
        } else {
            Self::Required { sep }
        })
    }
}

pub struct Arg {
    pub name: SpannedToken![ident],
    pub sep: Sep,
    // pub typ
}

impl Parse for Arg {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(todo!())
    }
}

impl Peek for Arg {
    fn is(token: &crate::tokens::tokens::SpannedToken) -> bool {
        <Token![ident]>::is(token)
    }
}

pub struct Struct {
    pub kw: SpannedToken![struct],
    pub name: SpannedToken![ident],
    brace: (),
    pub args: Repeated<Arg, Token![,]>,
}
