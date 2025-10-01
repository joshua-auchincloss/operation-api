pub mod builtin;
pub mod enums;
pub mod ident;
pub mod imports;
pub mod meta;
pub mod namespace;
pub mod oneof;
pub mod struct_def;
pub mod ty;
pub mod value;

pub mod payload;

pub use builtin::*;
pub use enums::*;
pub use ident::*;
pub use imports::*;
pub use meta::*;
pub use namespace::*;
pub use oneof::*;
pub use payload::*;
pub use struct_def::*;
pub use ty::*;
pub use value::*;

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

fn quoted_inner<'s>(value: Pair<'s, Rule>) -> &'s str {
    value.into_inner().next().unwrap().as_str()
}

const SINGLE_QUOTE: &str = "'";
const DOUBLE_QUOTE: &str = "\"";

fn clean_rawvalue(s: &str) -> String {
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
    #[allow(clippy::never_loop)]
    for comment in comment {
        return clean_rawstr(comment.as_str());
    }
    unreachable!()
}
pub trait Commentable {
    fn comment(
        &mut self,
        comment: String,
    );

    fn comment_pairs(
        &mut self,
        comment: Pairs<'_, Rule>,
    ) {
        self.comment(take_comment(comment))
    }
}

pub trait FromInner: Sized {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self>;
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.start == other.start && self.end == other.end && self.value == other.value
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: std::hash::Hash> std::hash::Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.start.hash(state);
        self.end.hash(state);
        self.value.hash(state);
    }
}

impl<T> Spanned<T> {
    pub fn new(
        start: usize,
        end: usize,
        value: T,
    ) -> Self {
        Self { start, end, value }
    }
    pub fn map<U>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Spanned<U> {
        Spanned {
            start: self.start,
            end: self.end,
            value: f(self.value),
        }
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub trait FromPairSpan: Sized {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>>;
}

#[allow(dead_code)]
pub(crate) fn spanned_from_pair<T: FromInner>(
    pair: pest::iterators::Pair<'_, Rule>
) -> crate::Result<Spanned<T>> {
    let span = pair.as_span();
    let start = span.start();
    let end = span.end();
    let inner = pair.into_inner();
    let value = T::from_inner(inner)?;
    Ok(Spanned::new(start, end, value))
}

fn apply_pending_if_forward<C: Commentable>(
    c: &mut C,
    pending: &mut Option<String>,
) {
    if let Some(comment) = pending {
        c.comment(comment.to_owned());
        *pending = None;
    }
}

#[inline]
fn apply_pending_meta<T>(
    target: &mut Vec<T>,
    pending: &mut Vec<T>,
) {
    if pending.is_empty() {
        return;
    }
    target.append(pending);
}
