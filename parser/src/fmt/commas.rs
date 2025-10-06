use crate::fmt::*;

dyn_inventory::emit! {
    NoTrailingCommaHandle AstFormat as Rule {
        meta = EditMeta {
            name: "no_trailing_comma_before_rbrace",
            description: "Do not include a trailing comma in the final field before `}`.",
            group: RuleGroup::TrailingCommas,
            level: RuleLevel::Warn,
            fix: Fix::Safe,
        }
    }
}

impl AstFormat for NoTrailingCommaHandle {
    fn analyze(
        &self,
        toks: &[SpannedToken],
    ) -> Vec<Edit> {
        let mut edits = Vec::new();
        for i in 0..toks.len() {
            if matches!(toks[i].value, Token::Comma) {
                if let Some(j) = next_non_ws(toks, i + 1) {
                    if matches!(toks[j].value, Token::RBrace) {
                        let span = Some(toks[i].span.clone());
                        edits.push(Edit {
                            kind: EditKind::Remove { range: i..i + 1 },
                            message: "remove trailing comma before `}`",
                            span,
                        });
                    }
                }
            }
        }
        edits
    }
}
