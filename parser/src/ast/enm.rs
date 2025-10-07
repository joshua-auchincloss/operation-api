use crate::tokens::{self, Brace, ToTokens};

use crate::{
    SpannedToken, Token,
    ast::comment::CommentStream,
    defs::Spanned,
    tokens::{ImplDiagnostic, MutTokenStream, Parse, Peek, Repeated, brace},
};

pub struct EnumValue<Value: Parse> {
    pub eq: SpannedToken![=],
    pub value: Spanned<Value>,
}

impl<Value: Parse> Peek for EnumValue<Value> {
    fn is(token: &crate::tokens::toks::Token) -> bool {
        <Token![=]>::is(token)
    }
}

impl<Value: Parse> Parse for EnumValue<Value> {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            eq: stream.parse()?,
            value: stream.parse()?,
        })
    }
}

impl<V: Parse + ToTokens> ToTokens for EnumValue<V> {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        tt.write(&self.eq);
        tt.write(&self.value);
    }
}

pub struct EnumVariant<Value: Parse> {
    pub comments: CommentStream,
    pub name: SpannedToken![ident],
    pub value: Option<EnumValue<Value>>,
}

impl<Value: Parse + Peek> Peek for EnumVariant<Value> {
    fn is(token: &crate::tokens::toks::Token) -> bool {
        <Token![ident]>::is(token)
    }
}

impl<Value: Parse + Peek> Parse for EnumVariant<Value> {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            comments: CommentStream::parse(stream)?,
            name: stream.parse()?,
            value: Option::parse(stream)?,
        })
    }
}

impl<Value: Parse + Peek> ImplDiagnostic for EnumVariant<Value> {
    fn fmt() -> &'static str {
        "enum variant"
    }
}

impl<V: Parse + Peek + ToTokens> ToTokens for EnumVariant<V> {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        tt.write(&self.comments);
        tt.write(&self.name);
        tt.write(&self.value);
    }
}

pub struct TypedEnum<Value: Parse + Peek> {
    pub kw: SpannedToken![enum],
    pub name: SpannedToken![ident],
    pub brace: Brace,
    pub variants: Repeated<EnumVariant<Value>, Token![,]>,
}

impl<Value: Parse + Peek + ImplDiagnostic> Parse for TypedEnum<Value> {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut brace;
        Ok(Self {
            kw: stream.parse()?,
            name: stream.parse()?,
            brace: brace!(brace in stream),
            variants: Repeated::parse(&mut brace)?,
        })
    }
}

impl<V: Parse + Peek + ToTokens> ToTokens for TypedEnum<V> {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        tt.write(&self.kw);
        tt.write(&self.name);
        tt.write(&tokens::LBraceToken::new());
        tt.write(&self.variants);
        tt.write(&tokens::RBraceToken::new());
    }
}

pub enum Enum {
    Int(TypedEnum<Token![number]>),
    Str(TypedEnum<Token![string]>),
}

impl Peek for Enum {
    fn is(token: &crate::tokens::toks::Token) -> bool {
        <Token![enum]>::is(token)
    }
}

impl Parse for Enum {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut f1 = stream.fork();

        let _: SpannedToken![enum] = f1.parse()?;
        let _: SpannedToken![ident] = f1.parse()?;

        let mut brace;
        brace!(
            brace in f1
        );

        Ok(
            if EnumVariant::<Token![number]>::parse(&mut brace).is_ok() {
                Self::Int(TypedEnum::parse(stream)?)
            } else {
                Self::Str(TypedEnum::parse(stream)?)
            },
        )
    }
}

impl ToTokens for Enum {
    fn write(
        &self,
        tt: &mut MutTokenStream,
    ) {
        match self {
            Self::Int(i) => {
                tt.write(&i.kw);
                tt.write(&i.name);
                tt.write(&tokens::LBraceToken::new());
                tt.write(&i.variants);
                tt.write(&tokens::RBraceToken::new());
            },
            Self::Str(s) => {
                tt.write(&s.kw);
                tt.write(&s.name);
                tt.write(&tokens::LBraceToken::new());
                tt.write(&s.variants);
                tt.write(&tokens::RBraceToken::new());
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::tokenize;

    use super::*;

    #[test_case::test_case(
    "enum Foo { Bar, Baz = 2 }", |it| {
        assert!(matches!(it, Enum::Int(..)))
    };
    "parses int enum with mixed eq and default values"
)]
    #[test_case::test_case(
    "enum Foo { Bar = \"a\", Baz = \"b\" }", |it| {
        assert!(matches!(it, Enum::Str(..)))
    };
    "parses str enum"
)]
    #[test_case::test_case(
    "enum Abc {Bar = 1}", |it| {
        assert!(matches!(it, Enum::Int(..)))
    };
    "parses enum variant with int value"
)]
    #[test_case::test_case(
    "enum Abc {Bar = \"a\"}", |it| {
        assert!(matches!(it, Enum::Str(..)))
    };
    "parses enum variant with str value"
)]
    fn test_enum_variant_parse_str(
        input: &str,
        matches: impl Fn(Enum),
    ) {
        let mut stream = tokenize(input).unwrap();
        let result = Enum::parse(&mut stream).unwrap();

        matches(result)
    }
}
