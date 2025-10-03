use logos::Logos;
use std::sync::Arc;

use crate::{
    defs::{Spanned, span::Span},
    tokens::{
        AstResult, ImplDiagnostic, NewlineToken,
        ast::{Parse, Peek},
        error::LexingError,
        tokens::{SpannedToken, Token},
    },
};

#[derive(Debug, Clone)]
pub struct TokenStream {
    pub(crate) source: Arc<str>,
    pub(crate) tokens: Arc<Vec<SpannedToken>>,
    pub(crate) cursor: usize, // absolute cursor
    range_start: usize,       // visible window start (absolute)
    range_end: usize,         // visible window end (exclusive, absolute)
}

macro_rules! shared_peek {
    () => {
        pub fn has_tokens_before_newline<T: Peek>(
            &self,
            pos: usize,
        ) -> bool {
            if let Some(current) = self.tokens.get(pos) {
                if T::is(current) {
                    return true;
                }
                for tok in self.tokens[..pos - 1].iter().rev() {
                    if NewlineToken::is(tok) {
                        return false;
                    } else if T::is(tok) {
                        return true;
                    }
                }
                false
            } else {
                false
            }
        }
    };
}

impl TokenStream {
    pub fn lex(source: &str) -> Result<Self, LexingError> {
        let source_rc: Arc<str> = Arc::from(source);
        let mut lex = Token::lexer(&source_rc);
        let mut toks = Vec::new();
        while let Some(token) = lex.next() {
            let token = token?;
            let span = lex.span();
            toks.push(Spanned::new(span.start, span.end, token));
        }
        let range_end = toks.len();
        Ok(Self {
            source: source_rc,
            tokens: Arc::new(toks),
            cursor: 0,
            range_start: 0,
            range_end,
        })
    }
    pub fn fork(&self) -> Self {
        Self {
            source: self.source.clone(),
            tokens: self.tokens.clone(),
            cursor: self.cursor,
            range_start: self.range_start,
            range_end: self.range_end,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.cursor >= self.range_end
    }

    pub fn peek_unchecked(&self) -> Option<&SpannedToken> {
        let mut cursor = self.cursor;
        let mut next = self.peek_unchecked_with_whitespace();
        while let Some(v) = next {
            if !matches!(v.value, Token::Newline) {
                return Some(v);
            } else {
                cursor += 1;
                next = self.tokens.get(cursor);
            }
        }
        None
    }

    pub fn peek_unchecked_with_whitespace(&self) -> Option<&SpannedToken> {
        if self.cursor < self.range_end {
            self.tokens.get(self.cursor)
        } else {
            None
        }
    }

    pub fn nth(
        &self,
        n: usize,
    ) -> Option<&SpannedToken> {
        let idx = self.cursor + n;
        if idx < self.range_end {
            self.tokens.get(idx)
        } else {
            None
        }
    }

    pub fn next_with_whitespace(&mut self) -> Option<SpannedToken> {
        if self.cursor >= self.range_end {
            return None;
        }
        let t = self.tokens.get(self.cursor).cloned();
        if t.is_some() {
            self.cursor += 1;
        }
        t
    }

    pub fn next(&mut self) -> Option<SpannedToken> {
        let mut next = self.next_with_whitespace();
        while let Some(v) = next {
            if !matches!(v.value, Token::Newline) {
                return Some(v);
            } else {
                tracing::trace!(cursor=%self.cursor, "skipping whitespace");
                next = self.next_with_whitespace();
            }
        }
        None
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn rewind(
        &mut self,
        to: usize,
    ) {
        let clamped = to.clamp(self.range_start, self.range_end);
        self.cursor = clamped;
    }
    pub fn len(&self) -> usize {
        self.range_end.saturating_sub(self.cursor)
    }
    pub fn all(&self) -> &[SpannedToken] {
        &self.tokens[self.range_start..self.range_end]
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn span_slice(
        &self,
        span: &Span,
    ) -> &str {
        &self.source[span.start..span.end]
    }

    pub fn parse<T: Parse>(&mut self) -> Result<Spanned<T>, LexingError> {
        T::parse_spanned(self)
    }

    pub fn peek<T: Peek>(&self) -> bool {
        T::peek(self)
    }

    pub fn span_of(
        &self,
        cursor: usize,
    ) -> Option<&Span> {
        if cursor < self.range_end {
            self.tokens.get(cursor).map(|t| &t.span)
        } else {
            None
        }
    }

    pub fn current_span(&self) -> &Span {
        self.span_of(self.cursor)
            .unwrap_or(&Span { start: 0, end: 0 })
    }

    pub fn last_span(&self) -> Option<&Span> {
        if self.cursor != 0 {
            self.span_of(self.cursor - 1)
        } else {
            None
        }
    }

    shared_peek! {}

    pub fn extract_inner_tokens<
        Open: Parse + Peek + ImplDiagnostic,
        Close: Parse + Peek + ImplDiagnostic,
    >(
        &mut self
    ) -> AstResult<(TokenStream, Spanned<()>)> {
        let (mut depth, first_span) = if let Some(first) = self.next() {
            if !Open::is(&first) {
                return Err(LexingError::expected::<Open>(first.value).with_span(first.span));
            }
            (1_usize, first.span)
        } else {
            return Err(LexingError::empty::<Open>());
        };

        let open_index = self.cursor - 1; // absolute index of opening token

        let mut end_pos = None;

        while let Some(tok) = self.next() {
            if Open::is(&tok) {
                depth += 1;
            } else if Close::is(&tok) {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        // matched close, after we have decremented the depth to 0
                        end_pos = Some(self.cursor);
                        break;
                    }
                }
            }
        }

        if let Some(end) = end_pos {
            // end is position AFTER closing token consumed
            let close_index = end - 1; // closing token
            let inner_start = open_index + 1;
            let inner_end = close_index; // exclusive range
            Ok((
                TokenStream {
                    source: self.source.clone(),
                    tokens: self.tokens.clone(),
                    cursor: inner_start,
                    range_start: inner_start,
                    range_end: inner_end,
                },
                Spanned::new(open_index, end, ()),
            ))
        } else {
            let mut err = LexingError::empty::<Close>();
            if let Some(last) = self.tokens.get(self.cursor - 1) {
                err = err.with_span(last.span.clone());
            } else {
                err = err.with_span(first_span)
            }
            Err(err)
        }
    }
}

pub fn tokenize(src: &str) -> Result<TokenStream, LexingError> {
    TokenStream::lex(src)
}

macro_rules! paired {
    ($tok: ident) => {
        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        pub struct $tok(Spanned<()>);

        impl $tok {
            pub fn new(span: Spanned<()>) -> Self {
                Self(span)
            }

            pub fn span(&self) -> &Span {
                &self.0.span
            }
        }

        paste::paste! {
            macro_rules! [<$tok:snake>] {
                (
                    $tokens: ident in $input: ident
                ) => {
                    match $input.extract_inner_tokens::<
                            $crate::tokens::tokens::[<L $tok:camel Token>],
                            $crate::tokens::tokens::[<R $tok:camel Token>],
                        >() {
                        Ok((token, span)) => {
                            $tokens = token;
                            $crate::tokens::$tok::new(span)
                        },
                        Err(e) => return Err(e)
                    }
                };
                (
                    $tokens: ident in $input: ident; $err: expr
                ) => {
                    match $input.extract_inner_tokens::<
                            $crate::tokens::tokens::[<L $tok:camel Token>],
                            $crate::tokens::tokens::[<R $tok:camel Token>],
                        >() {
                        Ok((token, span)) => {
                            $tokens = token;
                            $crate::tokens::$tok::new(span)
                        },
                        Err(..) => return $err
                    }
                };
            }
            pub(crate) use [<$tok:snake>];
        }
    };
}

paired! {
    Brace
}
paired! {
    Bracket
}
paired! {
    Paren
}

fn spanned_value(span: Spanned<Token>) -> Token {
    span.value
}

impl IntoIterator for TokenStream {
    type Item = Token;
    type IntoIter = <Vec<Token> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens[self.range_start..self.range_end]
            .iter()
            .cloned()
            .map(spanned_value)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[derive(Default, Debug)]
pub struct MutTokenStream {
    pub(crate) tokens: Vec<Token>,
}

impl MutTokenStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(sz: usize) -> Self {
        Self {
            tokens: Vec::with_capacity(sz),
        }
    }

    pub fn push(
        &mut self,
        token: Token,
    ) {
        self.tokens.push(token)
    }

    pub fn extend<I: IntoIterator<Item = Token>>(
        &mut self,
        i: I,
    ) {
        self.tokens
            .append(&mut i.into_iter().collect());
    }

    shared_peek! {}
}

impl IntoIterator for MutTokenStream {
    type Item = Token;
    type IntoIter = <Vec<Token> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}

impl std::fmt::Display for MutTokenStream {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if self.tokens.is_empty() {
            return Ok(());
        }

        let last = self.tokens.len() - 1;
        for (pos, tok) in self.tokens.iter().enumerate() {
            write!(f, "{tok}")?;

            let next = self.tokens.get(pos + 1);
            if matches!(
                tok,
                Token::CommentMultiLine(..) | Token::CommentSingleLine(..)
            ) {
                write!(f, "\n")?;
            } else if pos != last
                && !matches!(tok, Token::LBrace | Token::LBracket | Token::LParen)
                && !matches!(
                    next,
                    Some(Token::RBrace) | Some(Token::RBracket) | Some(Token::RParen)
                )
            {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::{
        defs::Spanned,
        tokens::{self, AstResult, Repeated, TokenStream, brace, bracket, paren, tokenize},
    };

    #[test_case::test_case(
        "{a}", |tt| {
            let inner;

            brace!(inner in tt);

            Ok(inner)
        }, 3; "parses braced with exact inner"
    )]
    #[test_case::test_case(
        "
            {
                a
            }
        ", |tt| {
            let inner;

            brace!(inner in tt);

            Ok(inner)
        }, 6; "parses braced"
    )]
    #[test_case::test_case(
        "
            [
                a
            ]
        ", |tt| {
            let inner;

            bracket!(inner in tt);

            Ok(inner)
        }, 6; "parses bracketed"
    )]
    #[test_case::test_case(
        "
            (
                a
            )
        ", |tt| {
            let inner;

            paren!(inner in tt);

            Ok(inner)
        }, 6; "parses parenthesized"
    )]
    fn test_paired(
        src: &str,
        get: impl Fn(&mut TokenStream) -> AstResult<TokenStream>,
        cursor: usize,
    ) -> crate::tokens::AstResult<()> {
        let mut tt = tokenize(src).unwrap();
        let mut inner = get(&mut tt)?;

        let a: Spanned<tokens::IdentToken> = inner.parse()?;
        assert_eq!(a.borrow_string(), "a");
        assert_eq!(tt.cursor, cursor);

        Ok(())
    }

    #[test_case::test_case(
        "
            }
    ",
        &["expected {, found }", "2:13"]; "close brace no open"
    )]
    #[test_case::test_case(
        "
        { a
    ",
        &["expected }, found end of token stream", "2:12"]; "open brace without close"
    )]
    fn test_paired_diagnostic(
        src: &str,
        expect: &[&str],
    ) {
        let mut tt = tokenize(src).expect("parse");
        let inner = || -> AstResult<TokenStream> {
            let inner;
            brace!(inner in tt);
            Ok(inner)
        }()
        .unwrap_err();
        let as_crate = crate::Error::from(inner);
        let p = Path::new("test.pld");
        let diag = format!("{:?}", as_crate.to_report_with(&p, src, None));
        eprintln!("{diag}");
        for e in expect {
            assert!(diag.contains(e), "'{}' is in ouputted diagnostics", e)
        }
    }
}
