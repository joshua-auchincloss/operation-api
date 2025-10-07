use crate::{
    SpannedToken, Token,
    ast::{self, comment::CommentStream, meta::ItemMeta},
    bail_unchecked,
    defs::Spanned,
    tokens::{ImplDiagnostic, LexingError, Parse, Peek, ToTokens, straight_through},
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

straight_through! {
    Item<T> {
        comments, meta, def, end
    }
}

pub type NamespaceDef = Item<super::namespace::Namespace>;
pub type SpannedNamespaceDef = Item<super::namespace::SpannedNamespace>;
pub type UseDef = Item<super::import::Use>;
pub type OneOfDef = Item<super::one_of::OneOf>;
pub type EnumDef = Item<super::enm::Enum>;
pub type StructDef = Item<super::strct::Struct>;
pub type TypeDef = Item<super::ty_def::NamedType>;
pub type ErrorDef = Item<super::err::ErrorType>;
pub type OperationDef = Item<super::op::Operation>;

pub enum Items {
    Use(UseDef),
    OneOf(OneOfDef),
    Enum(EnumDef),
    Struct(StructDef),
    Type(TypeDef),
    Error(ErrorDef),
    Operation(OperationDef),
    Namespace(NamespaceDef),
    SpannedNamespace(SpannedNamespaceDef),
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
        } else if stream.peek::<ast::import::Use>() {
            Self::Use(UseDef {
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
        } else if stream.peek::<ast::err::ErrorType>() {
            Self::Error(ErrorDef {
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
        } else if stream.peek::<ast::op::Operation>() {
            Self::Operation(OperationDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else if stream.peek::<ast::namespace::SpannedNamespace>() {
            Self::SpannedNamespace(SpannedNamespaceDef {
                comments,
                meta,
                def: stream.parse()?,
                end: stream.parse()?,
            })
        } else {
            let expect = vec![
                <Token![namespace]>::fmt(),
                <Token![use]>::fmt(),
                <Token![oneof]>::fmt(),
                <Token![enum]>::fmt(),
                <Token![struct]>::fmt(),
                <Token![error]>::fmt(),
                <Token![type]>::fmt(),
                <Token![operation]>::fmt(),
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
                | <Token![use]>::is(token)
                | <Token![oneof]>::is(token)
                | <Token![enum]>::is(token)
                | <Token![struct]>::is(token)
                | <Token![error]>::is(token)
                | <Token![type]>::is(token)
                | <Token![operation]>::is(token)
        } else {
            false
        }
    }
}

impl ToTokens for Items {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        use Items::*;
        match self {
            Use(def) => tt.write(def),
            OneOf(def) => tt.write(def),
            Enum(def) => tt.write(def),
            Struct(def) => tt.write(def),
            Type(def) => tt.write(def),
            Error(def) => tt.write(def),
            Operation(def) => tt.write(def),
            Namespace(def) => tt.write(def),
            SpannedNamespace(def) => tt.write(def),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::defs::Spanned;

    #[test_case::test_case(
        "
namespace test;

#[version(1)]
struct test_arrays {
    a: str[]
};", 2; "parses struct"
    )]
    #[test_case::test_case(
        "
namespace test;

#[version(1)]
oneof test_oneof {
    a(i32),
    b { desc: i32[] },
};", 2; "parses named oneof"
    )]
    #[test_case::test_case(
        "
namespace test;

#[version(1)]
error test_error {
    a(i32),
    b { desc: i32[] },
};", 2; "parses named error"
    )]
    #[test_case::test_case(
        "
namespace test;
use abc::foo;
", 2; "parses use"
    )]
    #[test_case::test_case(
        "
namespace test;

// an infallible operation
operation add(a: i32, b: i32) -> i32;
", 2; "parses 2 arg operation without result type"
    )]
    #[test_case::test_case(
        "
namespace test;

operation foo() -> i32!;
", 2; "parses 0 arg operation with result type"
    )]
    #[test_case::test_case(
        "
namespace test {
    operation foo() -> i32!;
};
", 1; "parses nested namespace"
    )]
    fn basic_smoke(
        src: &str,
        n_items: usize,
    ) {
        let items: Vec<Spanned<super::Items>> = crate::tst::basic_smoke(src);
        assert_eq!(items.len(), n_items);
    }
}
