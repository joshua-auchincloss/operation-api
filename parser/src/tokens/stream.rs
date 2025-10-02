use logos::Logos;
use std::{iter::Map, rc::Rc};

use crate::{
    defs::{Spanned, span::Span},
    tokens::{
        AstResult, ImplDiagnostic,
        ast::{Parse, Peek},
        error::LexingError,
        tokens::{SpannedToken, Token},
    },
};

#[derive(Debug, Clone)]
pub struct TokenStream {
    pub(crate) source: Rc<str>,
    pub(crate) tokens: Rc<Vec<SpannedToken>>,
    pub(crate) cursor: usize,
}

impl TokenStream {
    pub fn lex(source: &str) -> Result<Self, LexingError> {
        let source_rc: Rc<str> = Rc::from(source);
        let mut lex = Token::lexer(&source_rc);
        let mut toks = Vec::new();
        while let Some(token) = lex.next() {
            let token = token?;
            let span = lex.span();
            toks.push(Spanned::new(span.start, span.end, token));
        }
        Ok(Self {
            source: source_rc,
            tokens: Rc::new(toks),
            cursor: 0,
        })
    }
    pub fn fork(&self) -> Self {
        Self {
            source: self.source.clone(),
            tokens: self.tokens.clone(),
            cursor: self.cursor,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.cursor >= self.tokens.len()
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
        self.tokens.get(self.cursor)
    }

    pub fn nth(
        &self,
        n: usize,
    ) -> Option<&SpannedToken> {
        self.tokens.get(self.cursor + n)
    }

    pub fn next_with_whitespace(&mut self) -> Option<SpannedToken> {
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
        self.cursor = to.min(self.tokens.len());
    }
    pub fn len(&self) -> usize {
        self.tokens.len() - self.cursor
    }
    pub fn all(&self) -> &[SpannedToken] {
        &self.tokens
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

    pub fn extract_inner_tokens<
        Open: Parse + Peek + ImplDiagnostic,
        Close: Parse + Peek + ImplDiagnostic,
    >(
        self: &mut Self
    ) -> AstResult<(TokenStream, Spanned<()>)> {
        let (mut depth, first_span) = if let Some(first) = self.next() {
            if !Open::is(&first) {
                return Err(LexingError::expected::<Open>(first.value).with_span(first.span));
            }
            (1_usize, first.span)
        } else {
            return Err(LexingError::empty::<Open>());
        };

        let start_pos = self.cursor - 1;

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

        // todo: this should really be a reference to the inner tokens vs a clone op as below
        if let Some(end) = end_pos {
            // fork w/o opening and closing tokens
            let inner_tokens = Rc::new(self.tokens[start_pos + 1..end - 1].to_vec());
            Ok((
                TokenStream {
                    source: self.source.clone(),
                    tokens: inner_tokens,
                    cursor: 0,
                },
                Spanned::new(start_pos, end, ()),
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
        self.tokens[..]
            .to_vec()
            .into_iter()
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
            if pos != last
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
