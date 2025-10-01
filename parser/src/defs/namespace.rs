use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder)]
pub struct NamespaceDef {
    pub comment: String,
    pub ident: Vec<Ident>,
}

impl FromInner for NamespaceDef {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut comment = String::new();
        for p in pairs {
            match p.as_rule() {
                Rule::singleline_comment | Rule::multiline_comment => {
                    comment += &take_comment(Pairs::single(p));
                }
                Rule::ident => {
                    return Ok(Self::builder()
                        .ident(Ident::from_inner(Pairs::single(p))?.qualified_path())
                        .comment(comment)
                        .build());
                }
                _ => {}
            }
        }
        Err(crate::Error::def::<Self>(Rule::ident))
    }
}

impl Commentable for NamespaceDef {
    fn comment(&mut self, comment: String) {
        self.comment += &comment;
    }
}
