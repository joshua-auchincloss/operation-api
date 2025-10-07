use serde::{Deserialize, Serialize};
use std::ops::Range;

use crate::{
    Peek,
    defs::span::Span,
    tokens::{
        LBraceToken, LParenToken, MutTokenStream, RBraceToken, RParenToken, SpannedToken, Token,
        TokenStream,
    },
};
use operation_api_manifests::rules::*;

pub mod commas;
pub mod comments;
pub mod eof;
pub mod parens;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RuleGroup {
    PairedTokens,
    OneOf,
    TrailingCommas,
    Comments,

    File,
}

operation_api_manifests::rule_config! {
    "op-fmt" for RuleGroup
}

#[derive(Clone, Debug)]
pub struct Edit {
    pub kind: EditKind,
    pub message: &'static str,
    pub span: Option<Span>,
}

#[derive(Clone, Debug)]
pub enum EditKind {
    Replace {
        range: Range<usize>,
        with: Vec<Token>,
    },
    Insert {
        at: usize,
        tokens: Vec<Token>,
    },
    Remove {
        range: Range<usize>,
    },
}

impl EditKind {
    fn start_index(&self) -> usize {
        match self {
            EditKind::Replace { range, .. } => range.start,
            EditKind::Insert { at, .. } => *at,
            EditKind::Remove { range } => range.start,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub group: RuleGroup,
    pub level: RuleLevel,
    pub fix: Fix,
}

#[derive(Clone, Debug)]
pub struct Lint<'m> {
    pub edit: Edit,
    pub meta: &'m EditMeta,
}

pub trait AstFormat: Send + Sync {
    fn analyze(
        &self,
        tokens: &[SpannedToken],
    ) -> Vec<Edit>;
}

dyn_inventory::dyn_inventory! {
    Rule<T: AstFormat> {
        pub meta: EditMeta,
        pub handle: T
    }
}

impl RuleCollector {
    #[cfg(feature = "emit")]
    pub fn emit_rules(
        &self,
        path: impl AsRef<std::path::Path>,
    ) {
        let mut meta = vec![];
        for plugin in &self.plugins {
            meta.push(plugin.meta.clone());
        }
        let as_json = serde_json::to_string(&meta).unwrap();
        std::fs::write(path.as_ref(), as_json).unwrap();
    }

    pub fn format_tokens<'m>(
        &'m self,
        target: &mut MutTokenStream,
        source_for_spans: Option<&TokenStream>,
        dry: bool,
    ) -> Vec<Lint<'m>> {
        let owned;
        let view: &[SpannedToken] = if let Some(ts) = source_for_spans {
            ts.all()
        } else {
            owned = target
                .tokens
                .iter()
                .cloned()
                .map(|t| crate::defs::Spanned::new(0, 0, t))
                .collect::<Vec<_>>();
            &owned
        };

        let mut lints: Vec<Lint> = Vec::new();
        for rule in &self.plugins {
            let edits = rule.handle.analyze(view);
            lints.extend(edits.into_iter().map(|edit| {
                Lint {
                    edit,
                    meta: &rule.meta,
                }
            }));
        }
        lints.sort_by_key(|e| e.edit.kind.start_index());

        if !dry && !lints.is_empty() {
            apply_edits_in_place(target, &lints);
        }
        lints
    }
}

fn apply_edits_in_place(
    target: &mut MutTokenStream,
    lints: &[Lint],
) {
    let mut toks: Vec<Token> = std::mem::take(&mut target.tokens);
    let mut shift: isize = 0;
    for l in lints {
        match &l.edit.kind {
            EditKind::Insert { at, tokens } => {
                let at = (*at as isize + shift).max(0) as usize;
                toks.splice(at..at, tokens.clone());
                shift += tokens.len() as isize;
            },
            EditKind::Remove { range } => {
                let start = (range.start as isize + shift).max(0) as usize;
                let end = (range.end as isize + shift).max(start as isize) as usize;
                toks.drain(start..end);
                shift -= (range.end - range.start) as isize;
            },
            EditKind::Replace { range, with } => {
                let start = (range.start as isize + shift).max(0) as usize;
                let end = (range.end as isize + shift).max(start as isize) as usize;
                toks.splice(start..end, with.clone());
                shift += with.len() as isize - (range.end - range.start) as isize;
            },
        }
    }
    target.tokens = toks;
}

pub(crate) fn next_non_ws(
    tokens: &[SpannedToken],
    mut i: usize,
) -> Option<usize> {
    while i < tokens.len() {
        if matches!(tokens[i].value, Token::Newline) {
            i += 1;
        } else {
            return Some(i);
        }
    }
    None
}

pub(crate) fn prev_non_ws(
    tokens: &[SpannedToken],
    mut i: isize,
) -> Option<usize> {
    while i >= 0 {
        if matches!(tokens[i as usize].value, Token::Newline) {
            i -= 1;
        } else {
            return Some(i as usize);
        }
    }
    None
}

pub(crate) fn build_pairs_for<Open: Peek, Close: Peek>(
    tokens: &[SpannedToken]
) -> Vec<Option<usize>> {
    let mut open_stack: Vec<usize> = Vec::new();
    let mut pairs: Vec<Option<usize>> = vec![None; tokens.len()];

    for (i, t) in tokens.iter().enumerate() {
        if Open::is(t) {
            open_stack.push(i)
        } else if Close::is(t)
            && let Some(o) = open_stack.pop()
        {
            pairs[o] = Some(i);
        }
    }

    pairs
}

pub(crate) fn build_pairs(tokens: &[SpannedToken]) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    let paren_pairs = build_pairs_for::<LParenToken, RParenToken>(tokens);
    let brace_pairs = build_pairs_for::<LBraceToken, RBraceToken>(tokens);
    (paren_pairs, brace_pairs)
}
