use crate::{
    SpannedToken,
    ast::{
        comment::CommentStream,
        meta::{IntMeta, ItemMeta},
    },
    defs::Spanned,
    tokens::Parse,
};

pub struct Item<T: Parse> {
    comments: CommentStream,
    meta: Spanned<ItemMeta>,
    def: Spanned<T>,
    end: SpannedToken![;],
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
