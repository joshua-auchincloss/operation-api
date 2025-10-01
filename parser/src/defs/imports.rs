use crate::defs::*;

use pest::iterators::Pairs;

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder, PartialEq, Eq, Hash)]
pub struct ImportDef {
    pub comment: String,
    pub path: String,
    pub meta: Vec<Meta>,
}

impl Commentable for ImportDef {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl FromInner for ImportDef {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut comment = String::new();
        for it in pairs {
            match it.as_rule() {
                Rule::singleline_comment | Rule::multiline_comment => {
                    comment += &take_comment(Pairs::single(it));
                },
                Rule::quoted => {
                    return Ok(Self::builder()
                        .path(quoted_inner(it).into())
                        .comment(comment)
                        .meta(Vec::new())
                        .build());
                },
                _ => panic!("{it:#?}"),
            }
        }

        Err(crate::Error::defs::<Self, _>([
            Rule::import_def,
            Rule::quoted,
        ]))
    }
}

impl FromPairSpan for ImportDef {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = ImportDef::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}
