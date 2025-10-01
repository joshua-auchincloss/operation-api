use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder, PartialEq, Eq, Hash)]
pub struct ImportDef {
    pub comment: String,
    pub path: String,
}

impl Commentable for ImportDef {
    fn comment(&mut self, comment: String) {
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
                }
                Rule::quoted => {
                    return Ok(Self::builder()
                        .path(quoted_inner(it).into())
                        .comment(comment)
                        .build());
                }
                rules => panic!("{it:#?}"),
            }
        }

        Err(crate::Error::defs::<Self, _>([
            Rule::import_def,
            // Rule::import,
            Rule::quoted,
        ]))
    }
}
