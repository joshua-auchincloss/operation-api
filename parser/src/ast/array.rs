use crate::{
    SpannedToken, Token,
    ast::ty::Type,
    defs::Spanned,
    tokens::{
        Bracket, ImplDiagnostic, MutTokenStream, Parse, Peek, ToTokens, Token, bracket, toks,
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
        tracing::trace!(cursor=%stream.cursor(), "parsing array");
        let ty = Box::new(stream.parse()?);
        let mut inner;
        let bracket = bracket!(inner in stream);
        Ok(if inner.peek::<Token![number]>() {
            tracing::trace!("parsing sized array");
            let size = inner.parse()?;
            Self::Sized { ty, bracket, size }
        } else {
            tracing::trace!("parsing unsized array");
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
        fork.peek::<toks::LBraceToken>()
    }
}

impl ToTokens for Array {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        match self {
            Self::Unsized { ty, .. } => {
                ty.write(tt);
                tt.push(Token::LBracket);
                tt.push(Token::RBracket);
            },
            Self::Sized { ty, size, .. } => {
                ty.write(tt);
                tt.push(Token::LBracket);
                size.write(tt);
                tt.push(Token::RBracket);
            },
        }
    }
}
