use crate::fmt::*;

dyn_inventory::emit! {
    EofHandle AstFormat as Rule {
        meta = EditMeta {
            name: "eof_should_terminate_with_newline",
            description: "Files should be terminated with a single new line.",
            group: RuleGroup::File,
            level: RuleLevel::Info,
            fix: Fix::Safe,
        }
    }
}

impl AstFormat for EofHandle {
    fn analyze(
        &self,
        tokens: &[SpannedToken],
    ) -> Vec<Edit> {
        let mut edits = Vec::new();

        let count = tokens
            .iter()
            .rev()
            .filter(|it| matches!(it.value, Token::Newline))
            .count();

        if count == 0 {
            edits.push(Edit {
                kind: EditKind::Insert {
                    at: tokens.len(),
                    tokens: vec![Token::Newline],
                },
                message: "add a new line to the end of the file",
                span: tokens.last().map(|it| it.span.clone()),
            })
        } else if count > 1 {
            let first_nl = tokens.len() - (count + 1);
            edits.push(Edit {
                kind: EditKind::Remove {
                    range: (first_nl..tokens.len()),
                },
                message: "remove excess new lines",
                span: Some(Span::new(
                    tokens[first_nl].span.start,
                    tokens[tokens.len()].span.end,
                )),
            })
        }

        edits
    }
}
