use logos::Logos;
use std::rc::Rc;

use crate::{
    defs::{Spanned, span::Span},
    tokens::{
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
}

pub fn tokenize(src: &str) -> Result<TokenStream, LexingError> {
    TokenStream::lex(src)
}
