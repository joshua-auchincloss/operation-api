use crate::{
    SpannedToken, Token,
    ast::AstStream,
    bail_unchecked,
    defs::Spanned,
    tokens::{self, Brace, LBraceToken, Parse, Peek, ToTokens, brace, straight_through},
};

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

impl Peek for Namespace {
    fn peek(stream: &tokens::TokenStream) -> bool {
        stream.peek::<Token![namespace]>() && !stream.peek::<SpannedNamespace>()
    }
}

straight_through! {
    Namespace {
        kw, name
    }
}

pub struct SpannedNamespace {
    pub kw: SpannedToken![namespace],
    pub name: SpannedToken![ident],
    pub brace: Brace,
    pub ast: Spanned<AstStream>,
}

impl Peek for SpannedNamespace {
    fn peek(stream: &tokens::TokenStream) -> bool {
        let mut fork = stream.fork();

        let _: SpannedToken![namespace] = bail_unchecked!(fork.parse(); false);
        let _: SpannedToken![ident] = bail_unchecked!(fork.parse(); false);

        fork.peek::<LBraceToken>()
    }
}

impl Parse for SpannedNamespace {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut braced;
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
            brace: brace!(braced in stream),
            ast: braced.parse()?,
        })
    }
}

impl ToTokens for SpannedNamespace {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        tt.write(&self.kw);
        tt.write(&self.name);
        tt.write(&tokens::LBraceToken::new());
        tt.write(&self.ast);
        tt.write(&tokens::RBraceToken::new());
    }
}
