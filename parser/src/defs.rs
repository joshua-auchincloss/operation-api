pub mod builtin;
pub mod enums;
pub mod ident;
pub mod imports;
pub mod message;
pub mod namespace;
pub mod ty;
pub mod union;
pub mod value;

pub mod payload;

pub use builtin::*;
pub use enums::*;
pub use ident::*;
pub use imports::*;
pub use message::*;
pub use namespace::*;
pub use payload::*;
pub use ty::*;
pub use union::*;
pub use value::*;

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

fn quoted_inner<'s>(value: Pair<'s, Rule>) -> &'s str {
    value.into_inner().next().unwrap().as_str()
}

const SINGLE_QUOTE: &str = "'";
const DOUBLE_QUOTE: &str = "\"";

fn clean_rawvalue<'s>(s: &str) -> String {
    s.trim_start_matches(SINGLE_QUOTE)
        .trim_end_matches(SINGLE_QUOTE)
        .trim_start_matches(DOUBLE_QUOTE)
        .trim_end_matches(DOUBLE_QUOTE)
        .to_string()
}

fn clean_rawstr(s: &str) -> String {
    s.trim()
        .trim_start_matches("/*")
        .trim_end_matches("*/")
        .trim_start_matches("//")
        .replace('\r', "")
        .replace("  ", " ")
        .trim_start_matches(" ")
        .trim_start_matches("\n")
        .trim_end_matches("\t")
        .trim_end_matches("\n")
        .to_string()
}

fn take_comment(comment: Pairs<'_, Rule>) -> String {
    for comment in comment {
        return clean_rawstr(comment.as_str());
    }
    unreachable!()
}
pub trait Commentable {
    fn comment(&mut self, comment: String);

    fn comment_pairs(&mut self, comment: Pairs<'_, Rule>) {
        self.comment(take_comment(comment))
    }
}

pub trait FromInner: Sized {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self>;
}

fn apply_pending_if_forward<C: Commentable>(c: &mut C, pending: &mut Option<String>) {
    match pending {
        Some(comment) => {
            c.comment(comment.to_owned());
            *pending = None;
        }
        _ => {}
    }
}
