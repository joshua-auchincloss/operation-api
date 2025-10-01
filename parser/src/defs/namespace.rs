use crate::defs::*;

use pest::iterators::Pairs;

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder)]
pub struct NamespaceDef<V> {
    pub comment: String,
    pub ident: Vec<Ident>,
    pub meta: Vec<Meta>,
    pub version: V,
}

impl<V: Default> FromInner for NamespaceDef<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut comment = String::new();
        for p in pairs {
            match p.as_rule() {
                Rule::singleline_comment | Rule::multiline_comment => {
                    comment += &take_comment(Pairs::single(p));
                },
                Rule::ident => {
                    let sp_ident = Ident::from_pair_span(p)?;
                    return Ok(Self::builder()
                        .ident(sp_ident.value.qualified_path())
                        .comment(comment)
                        .meta(Vec::new())
                        .version(V::default())
                        .build());
                },
                _ => {},
            }
        }
        Err(crate::Error::def::<Self>(Rule::ident))
    }
}

impl<V> Commentable for NamespaceDef<V> {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl<V: Default> FromPairSpan for NamespaceDef<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = NamespaceDef::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

pub type NamespaceSealed = NamespaceDef<usize>;
pub type NamespaceUnsealed = NamespaceDef<Option<usize>>;

impl NamespaceUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> NamespaceSealed {
        NamespaceDef {
            comment: self.comment,
            ident: self.ident,
            meta: self.meta,
            version: self.version.unwrap_or(file_version),
        }
    }
}
