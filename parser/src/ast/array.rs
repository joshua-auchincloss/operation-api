use crate::{
    SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{
        Bracket, ImplDiagnostic, MutTokenStream, Parse, Peek, SpannedToken, ToTokens, Token,
        bracket, tokens,
    },
};

/// array of form `i32[]` (unsized) or `i32[4]` (sized)
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Array {
    // i32[]
    Unsized {
        ty: Box<Spanned<Type>>,
        bracket: Bracket,
    },
    // i32[4]
    Sized {
        ty: Box<Spanned<Type>>,
        bracket: Bracket,
        size: SpannedToken![number],
    },
}

impl ImplDiagnostic for Array {
    fn fmt() -> &'static str {
        "i32[] | i32[4]"
    }
}

impl Parse for Array {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let ty = Box::new(stream.parse()?);
        let mut inner;
        let bracket = bracket!(inner in stream);
        Ok(if inner.peek::<Token![number]>() {
            let size = inner.parse()?;
            Self::Sized { ty, bracket, size }
        } else {
            Self::Unsized { ty, bracket }
        })
    }
}

impl Peek for Array {
    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        let mut fork = stream.fork();
        if fork.parse::<Spanned<Type>>().is_err() {
            return false;
        };
        fork.peek::<tokens::LBraceToken>()
    }
}

impl ToTokens for Array {
    fn tokens(&self) -> MutTokenStream {
        let mut tt = MutTokenStream::new();
        match self {
            Self::Unsized { ty, .. } => {
                ty.write(&mut tt);
                tt.push(Token::LBracket);
                tt.push(Token::RBracket);
            },
            Self::Sized { ty, size, .. } => {
                ty.write(&mut tt);
                tt.push(Token::LBracket);
                size.write(&mut tt);
                tt.push(Token::RBracket);
            },
        }
        tt
    }
}
