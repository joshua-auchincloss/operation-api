use crate::{
    SpannedToken, Token, bail_unchecked,
    defs::Spanned,
    tokens::{self, Bracket, MutTokenStream, Paren, Parse, Peek, ToTokens, bracket, paren},
};

pub struct Meta<Value: Parse> {
    pub open: SpannedToken![#],
    pub inner: Option<Token![!]>,
    pub bracket: Bracket,
    pub name: SpannedToken![ident],
    pub paren: Paren,
    pub value: Spanned<Value>,
}

impl<Value: Parse + Peek> tokens::Peek for Meta<Value> {
    fn peek(stream: &tokens::TokenStream) -> bool {
        let mut stream = stream.fork();

        let mut bracket;
        let mut paren;
        let open = bail_unchecked!(stream.parse(); false);

        let inner = bail_unchecked!(Option::parse(&mut stream); false);

        let bracket_tok = bracket!(bracket in stream; false);

        let name = bail_unchecked!(bracket.parse(); false);

        let paren_tok = paren!(paren in bracket; false);

        let _ = Self {
            open,
            inner,
            bracket: bracket_tok,
            name,
            paren: paren_tok,
            value: bail_unchecked!(paren.parse(); false),
        };

        true
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

impl<Value: Parse + Peek + ToTokens> ToTokens for Meta<Value> {
    fn tokens(&self) -> MutTokenStream {
        let mut tt = MutTokenStream::new();

        tt.write(&self.open);
        tt.write(&self.inner);

        tt.write(&tokens::LBracketToken::new());
        tt.write(&self.name);
        tt.write(&tokens::LParenToken::new());
        tt.write(&self.value);
        tt.write(&tokens::RParenToken::new());
        tt.write(&tokens::RBracketToken::new());

        tt
    }
}

pub type IntMeta = Meta<Token![number]>;
pub type StrMeta = Meta<Token![string]>;
pub type IdentMeta = Meta<Token![ident]>;

pub enum ItemMetaItem {
    Version(Spanned<IntMeta>),
    Error(Spanned<IdentMeta>),
}

pub struct ItemMeta {
    pub meta: Vec<ItemMetaItem>,
}

impl Parse for ItemMeta {
    fn parse(stream: &mut tokens::TokenStream) -> Result<Self, tokens::LexingError> {
        let mut meta = vec![];
        loop {
            if stream.peek::<IntMeta>() {
                let this: Spanned<IntMeta> = stream.parse()?;
                match this.name.borrow_string().as_ref() {
                    "version" => meta.push(ItemMetaItem::Version(this)),
                    unknown => {
                        return Err(crate::LexingError::unknown_meta(
                            vec!["version"],
                            unknown.into(),
                            &this.name.span,
                        ));
                    },
                }
            } else if stream.peek::<StrMeta>() {
                let this: Spanned<StrMeta> = stream.parse()?;
                match this.name.borrow_string() {
                    unknown => {
                        return Err(crate::LexingError::unknown_meta(
                            vec![],
                            unknown.into(),
                            &this.name.span,
                        ));
                    },
                }
            } else if stream.peek::<IdentMeta>() {
                let this: Spanned<IdentMeta> = stream.parse()?;
                match this.name.borrow_string().as_ref() {
                    "error" => meta.push(ItemMetaItem::Error(this)),
                    unknown => {
                        return Err(crate::LexingError::unknown_meta(
                            vec!["error"],
                            unknown.into(),
                            &this.name.span,
                        ));
                    },
                }
            } else {
                break;
            }
        }

        Ok(Self { meta })
    }
}

impl ToTokens for ItemMetaItem {
    fn tokens(&self) -> MutTokenStream {
        let mut tt = MutTokenStream::new();
        match self {
            ItemMetaItem::Version(m) => tt.write(m),
            ItemMetaItem::Error(m) => tt.write(m),
        }
        tt
    }
}

impl ToTokens for ItemMeta {
    fn tokens(&self) -> MutTokenStream {
        let mut tt = MutTokenStream::new();
        for m in &self.meta {
            tt.write(m);
        }
        tt
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use crate::{
        Error,
        tokens::{AstResult, Parse, tokenize},
    };

    use super::*;

    fn parse_meta<Value: Parse + Peek>(input: &str) -> AstResult<Meta<Value>> {
        let mut stream = tokenize(input)?;
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

    #[test_case::test_case("#[version(1)]", 1; "version from outer")]
    #[test_case::test_case("#![version(2)]", 2; "version from inner")]
    fn test_item_parse(
        src: &str,
        expect_version: i32,
    ) {
        let mut tt = tokenize(src).expect("Should parse");
        let meta: Spanned<ItemMeta> = tt.parse().unwrap();
        let version = match meta.meta.get(0).unwrap() {
            ItemMetaItem::Version(ver) => ver,
            #[allow(unreachable_patterns)]
            _ => panic!("not version"),
        };
        assert_eq!(*version.value.value.borrow_i32(), expect_version);
    }

    #[test_case::test_case("#[unknown(1)]", vec![
        "unknown meta attribute, 'unknown'. expected one of version",
        "1:3"
    ]; "unknown int meta")]
    #[test_case::test_case("#[foo(\"bar\")]", vec![
        "unknown meta attribute, 'foo'. expected one of version",
        "1:3"
    ]; "unknown string meta")]
    #[test_case::test_case("#![baz(42)]", vec![
        "unknown meta attribute, 'baz'. expected one of version",
        "1:4"
    ]; "unknown int meta with bang")]
    #[test_case::test_case("#![qux(\"val\")]", vec![
        "unknown meta attribute, 'qux'. expected one of version",
        "1:4"
    ]; "unknown string meta with bang")]
    fn test_unknown_meta(
        src: &str,
        expected_diag: Vec<&str>,
    ) {
        let mut tt = tokenize(src).expect("Should tokenize");
        let p = std::path::Path::new("foo.pld");
        let err = (match tt
            .parse::<Spanned<ItemMeta>>()
            .map_err(Error::from)
        {
            Ok(_) => panic!("ok"),
            Err(e) => e,
        })
        .to_report_with(p, &tt.source, None);
        let err = format!("{err:?}");
        eprintln!("{err}");

        for d in expected_diag {
            assert!(
                err.contains(d),
                "Expected diagnostic to contain:\n{}\nActual diagnostic:\n{}",
                d,
                err
            );
        }
    }
}
