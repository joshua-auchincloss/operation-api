use crate::{
    SpannedToken,
    ast::{comment::CommentStream, meta::IntMeta},
    defs::Spanned,
    tokens::Parse,
};

pub struct Definition<T: Parse> {
    comments: CommentStream,
    version: Option<IntMeta>,
    def: Spanned<T>,
    end: SpannedToken![;],
}

impl<T: Parse> Parse for Definition<T> {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        Ok(Self {
            comments: CommentStream::parse(stream)?,
            version: Option::parse(stream)?,
            def: stream.parse()?,
            end: stream.parse()?,
        })
    }
}

pub type NamespaceDef = Definition<super::namespace::Namespace>;
pub type ImportDef = Definition<super::import::Import>;
pub type OneOfDef = Definition<super::one_of::OneOf>;
pub type EnumDef = Definition<super::enm::Enum>;
