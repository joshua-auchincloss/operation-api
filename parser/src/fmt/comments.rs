use crate::fmt::*;

dyn_inventory::emit! { NewlineAfterSLHandle AstFormat as Rule {
    meta = EditMeta {
        name: "newline_after_single_line_comment",
        description: "Terminate single-line comments with a newline token.",
        group: RuleGroup::Comments,
        level: RuleLevel::Info,
        fix: Fix::Safe,
    }
}}

impl AstFormat for NewlineAfterSLHandle {
    fn analyze(
        &self,
        toks: &[SpannedToken],
    ) -> Vec<Edit> {
        let mut edits = Vec::new();
        for i in 0..toks.len() {
            if matches!(toks[i].value, Token::CommentSingleLine(_)) {
                let next = toks.get(i + 1).map(|t| &t.value);
                if !matches!(next, Some(Token::Newline)) {
                    let span = Some(toks[i].span.clone());
                    edits.push(Edit {
                        kind: EditKind::Insert {
                            at: i + 1,
                            tokens: vec![Token::Newline],
                        },
                        message: "insert newline after single-line comment",
                        span,
                    });
                }
            }
        }
        edits
    }
}
