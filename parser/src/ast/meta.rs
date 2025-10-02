use crate::{
    SpannedToken, Token,
    defs::Spanned,
    tokens::{self, Parse, Peek, Token, brace, bracket, paren},
};

pub struct Meta<Value: Parse> {
    open: SpannedToken![#],
    inner: Option<Token![!]>,
    bracket: (),
    name: SpannedToken![ident],
    paren: (),
    value: Spanned<Value>,
}

impl<Value: Parse> tokens::Peek for Meta<Value> {
    fn is(token: &tokens::tokens::SpannedToken) -> bool {
        <Token![#]>::is(token)
    }
}

impl<Value: Parse + Peek> tokens::Parse for Meta<Value> {
    fn parse(stream: &mut tokens::TokenStream) -> Result<Self, tokens::LexingError> {
        let mut bracket;
        let mut paren;
        Ok(Self {
            open: stream.parse()?,
            inner: Option::parse(stream)?,
            bracket: bracket!(bracket in stream),
            name: {
                println!("{bracket:#?}");
                bracket.parse()?
            },
            paren: paren!(paren in bracket),
            value: paren.parse()?,
        })
    }
}

pub type IntMeta = Meta<Token![number]>;
pub type StrMeta = Meta<Token![string]>;

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use crate::tokens::{AstResult, Parse, tokenize};

    use super::*;

    fn parse_meta<Value: Parse + Peek>(input: &str) -> AstResult<Meta<Value>> {
        let mut stream = tokenize(input)?;
        println!("{:#?}", stream);
        Meta::parse(&mut stream)
    }

    #[test_case::test_case("#[abc(\"value\")]", "abc", tokens::StringToken::new("value".into()) ; "string value")]
    #[test_case::test_case("#[abc(1)]", "abc", tokens::NumberToken::new(1) ; "int value")]
    #[test_case::test_case("#![abc(\"value\")]", "abc", tokens::StringToken::new("value".into()) ; "string value with bang")]
    #[test_case::test_case("#![abc(42)]", "abc", tokens::NumberToken::new(42) ; "int value with bang")]
    fn test_meta_parse<Value: Peek + Parse + PartialEq + Debug>(
        input: &str,
        expected_name: &str,
        expected_value: Value,
    ) {
        let meta: Meta<Value> = parse_meta(input).expect("Should parse");
        assert_eq!(meta.name.value.borrow_string(), expected_name);
        assert_eq!(meta.value.value, expected_value);
    }
}
