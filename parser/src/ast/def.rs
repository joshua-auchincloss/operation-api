use crate::{
    SpannedToken, Token,
    ast::{self, comment::CommentStream, meta::ItemMeta},
    bail_unchecked,
    defs::Spanned,
    tokens::{ImplDiagnostic, LexingError, Parse, Peek},
};

pub struct Item<T: Parse> {
    pub comments: CommentStream,
    pub meta: Spanned<ItemMeta>,
    pub def: Spanned<T>,
    pub end: SpannedToken![;],
}

impl<T: Parse> Parse for Item<T> {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            comments: CommentStream::parse(stream)?,
            meta: stream.parse()?,
            def: stream.parse()?,
            end: stream.parse()?,
        })
    }
}

pub type NamespaceDef = Item<super::namespace::Namespace>;
pub type ImportDef = Item<super::import::Import>;
pub type OneOfDef = Item<super::one_of::OneOf>;
pub type EnumDef = Item<super::enm::Enum>;
pub type StructDef = Item<super::strct::Struct>;
pub type TypeDef = Item<super::ty_def::NamedType>;

pub enum Items {
    Namespace(NamespaceDef),
    Import(ImportDef),
    OneOf(OneOfDef),
    Enum(EnumDef),
    Struct(StructDef),
    Type(TypeDef),
}

impl Parse for Items {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let comments = CommentStream::parse(stream)?;
        let meta = stream.parse()?;

        Ok(if stream.peek::<ast::namespace::Namespace>() {
            Self::Namespace(NamespaceDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::import::Import>() {
            Self::Import(ImportDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::one_of::OneOf>() {
            Self::OneOf(OneOfDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::enm::Enum>() {
            Self::Enum(EnumDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::strct::Struct>() {
            Self::Struct(StructDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::ty_def::NamedType>() {
            Self::Type(TypeDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else {
            let expect = vec![
                <Token![namespace]>::fmt(),
                <Token![import]>::fmt(),
                <Token![oneof]>::fmt(),
                <Token![enum]>::fmt(),
                <Token![struct]>::fmt(),
                <Token![error]>::fmt(),
                <Token![type]>::fmt(),
            ];
            return Err(if let Some(next) = stream.next() {
                LexingError::expected_oneof(expect, next.value)
            } else {
                LexingError::empty_oneof(expect)
            });
        })
    }
}

impl Peek for Items {
    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        let mut fork = stream.fork();
        let _ = bail_unchecked!(CommentStream::parse(&mut fork); false);
        let _: Spanned<ItemMeta> = bail_unchecked!(fork.parse(); false);
        if let Some(token) = fork.next() {
            let token = &token.value;
            <Token![namespace]>::is(token)
                | <Token![import]>::is(token)
                | <Token![oneof]>::is(token)
                | <Token![enum]>::is(token)
                | <Token![struct]>::is(token)
                | <Token![error]>::is(token)
                | <Token![type]>::is(token)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        defs::Spanned,
        tokens::{Parse, tokenize},
    };

    #[test_case::test_case(
        "
namespace test;

#[version(1)]
struct test_arrays {
    a: str[]
};", 2; "parses struct & namespace"
    )]
    fn basic_smoke(
        src: &str,
        n_items: usize,
    ) {
        crate::tst::logging();

        let mut tt = tokenize(src).unwrap();
        println!("{tt:#?}");
        let items: Vec<Spanned<super::Items>> = Vec::parse(&mut tt).unwrap();
        assert_eq!(items.len(), n_items);
    }
}
