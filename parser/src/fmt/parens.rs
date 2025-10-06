mod anon_struct {
    use crate::fmt::*;
    dyn_inventory::emit! { ParensAnonHandle AstFormat as Rule {
        meta = EditMeta {
            name: "parens_around_anonymous_struct",
            description: "Do not parenthesize anonymous structs. Prefer `{ ... }` over `({ ... })`.",
            group: RuleGroup::PairedTokens,
            level: RuleLevel::Warn,
            fix: Fix::Safe,
        }
    }}
    impl AstFormat for ParensAnonHandle {
        fn analyze(
            &self,
            toks: &[SpannedToken],
        ) -> Vec<Edit> {
            let (paren_pairs, brace_pairs) = build_pairs(toks);
            let mut edits = Vec::new();
            let mut i = 0usize;
            while i < toks.len() {
                if matches!(toks[i].value, Token::LParen) {
                    if let Some(lbrace) = next_non_ws(toks, i + 1) {
                        if matches!(toks[lbrace].value, Token::LBrace) {
                            if let (Some(rparen), Some(rbrace)) =
                                (paren_pairs[i], brace_pairs[lbrace])
                            {
                                if let Some(after_rbrace) = next_non_ws(toks, rbrace + 1) {
                                    if after_rbrace == rparen {
                                        let span = Some(Span::new(
                                            toks[i].span.start,
                                            toks[rparen].span.end,
                                        ));
                                        edits.push(Edit {
                                            kind: EditKind::Remove { range: i..i + 1 },
                                            message: "remove parentheses around anonymous struct",
                                            span: span.clone(),
                                        });
                                        edits.push(Edit {
                                            kind: EditKind::Remove {
                                                range: rparen..rparen + 1,
                                            },
                                            message: "remove parentheses around anonymous struct",
                                            span,
                                        });
                                        i = rparen + 1;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
                i += 1;
            }
            edits
        }
    }
}

mod oneof_array {
    use crate::fmt::*;
    dyn_inventory::emit! { ParensOneofArrayHandle AstFormat as Rule {
        meta = EditMeta {
            name: "parens_around_oneof_when_array",
            description: "Parenthesize `oneof` when applying array suffix, e.g., `(oneof a | b)[]`.",
            group: RuleGroup::OneOf,
            level: RuleLevel::Warn,
            fix: Fix::Safe,
        }
    }}

    impl AstFormat for ParensOneofArrayHandle {
        fn analyze(
            &self,
            toks: &[SpannedToken],
        ) -> Vec<Edit> {
            let mut edits = Vec::new();
            let mut i = 0usize;
            while i < toks.len() {
                if matches!(toks[i].value, Token::KwOneof) {
                    let prev = prev_non_ws(toks, i as isize - 1);
                    let already_parened = prev
                        .map(|p| matches!(toks[p].value, Token::LParen))
                        .unwrap_or(false);

                    // find end of oneof expression up to next significant boundary
                    let mut end = i + 1;
                    let mut depth_paren = 0isize;
                    let mut depth_brace = 0isize;
                    while end < toks.len() {
                        match toks[end].value {
                            Token::LParen => depth_paren += 1,
                            Token::RParen => {
                                if depth_paren == 0 {
                                    break;
                                }
                                depth_paren -= 1;
                            },
                            Token::LBrace => depth_brace += 1,
                            Token::RBrace => {
                                if depth_brace == 0 {
                                    break;
                                }
                                depth_brace -= 1;
                            },
                            Token::Comma | Token::Semi | Token::RBracket => break,
                            _ => {},
                        }
                        end += 1;
                    }
                    let after = next_non_ws(toks, end);
                    let needs_paren =
                        matches!(after.map(|j| &toks[j].value), Some(Token::LBracket));
                    if needs_paren && !already_parened {
                        let span = Some(toks[i].span.clone());
                        edits.push(Edit {
                            kind: EditKind::Insert {
                                at: i,
                                tokens: vec![Token::LParen],
                            },
                            message: "add parentheses around oneof before array suffix",
                            span: span.clone(),
                        });
                        let insert_pos = after.unwrap_or(end);
                        edits.push(Edit {
                            kind: EditKind::Insert {
                                at: insert_pos,
                                tokens: vec![Token::RParen],
                            },
                            message: "add parentheses around oneof before array suffix",
                            span,
                        });
                    }
                }
                i += 1;
            }
            edits
        }
    }
}
