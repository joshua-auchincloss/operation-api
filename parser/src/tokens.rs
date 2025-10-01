use crate::defs::span::Span;
use logos::Logos;
use std::{fmt, rc::Rc};

use crate::defs::Spanned;
pub use ast::*;
pub use error::*;
pub use stream::*;
pub use tokens::*;

pub trait ImplDiagnostic {
    fn fmt() -> &'static str;
}

pub mod error {
    use std::{
        num::{ParseFloatError, ParseIntError},
        str::ParseBoolError,
        string::ParseError,
    };

    use crate::{
        defs::span::Span,
        tokens::{ImplDiagnostic, Token},
    };

    #[derive(thiserror::Error, Debug, Clone, PartialEq, Default)]
    pub enum LexingError {
        #[default]
        #[error("unknown lexing error")]
        Unknown,

        #[error("parse int error: {0}")]
        ParseInt(#[from] ParseIntError),
        #[error("parse bool error: {0}")]
        ParseBool(#[from] ParseBoolError),
        #[error("parse float error: {0}")]
        ParseFloat(#[from] ParseFloatError),

        #[error("parse error: {0}")]
        Parse(#[from] ParseError),

        #[error("expected {expect}, found end of token stream")]
        EmptyTokens { expect: &'static str },

        #[error("expected {expect}, found '{found}'")]
        ExpectationFailure { expect: &'static str, found: Token },

        #[error("{source}")]
        Spanned { source: Box<Self>, span: Span },
    }

    impl LexingError {
        pub fn empty<D: ImplDiagnostic>() -> Self {
            Self::EmptyTokens { expect: D::fmt() }
        }

        pub fn expected<D: ImplDiagnostic>(found: Token) -> Self {
            Self::ExpectationFailure {
                expect: D::fmt(),
                found,
            }
        }

        pub fn with_span(
            self,
            span: Span,
        ) -> Self {
            Self::Spanned {
                source: Box::new(self),
                span,
            }
        }

        pub fn then_with_span(span: Span) -> impl FnOnce(Self) -> Self {
            |this| this.with_span(span)
        }
    }
}

pub mod tokens {
    use super::*;
    use crate::defs::span::Span;

    macro_rules! tokens {
        (
            $(
                $(#[token($met:literal)])*
                $(#[regex($reg:literal $(,$e:expr)?)])*
                $(#[regfmt($fmt:expr)])?
                $tok:ident $(($inner:ty))?
            ),+ $(,)?
        ) => {
            #[derive(Logos, Clone, PartialEq)]
            #[logos(skip r"[ \t\r\f]+")]
            #[logos(error = LexingError)]
            pub enum Token {
                $(
                    $(#[token($met)])*
                    $(#[regex($reg $(,$e)?)])*
                    $tok $(($inner))?,
                )*
            }

            paste::paste!{
                $(
                    #[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
                    pub struct [<$tok Token>](
                        $(
                            $inner,
                        )*
                        #[serde(skip)]
                        ()
                    );

                    impl [<$tok Token>]{
                        pub fn new($([<$inner:snake>]: $inner ,)*)-> Self {
                            Self($([<$inner:snake>],)*())
                        }

                        $(
                            pub fn [<borrow_ $inner:snake>](&self) -> &$inner {
                                &self.0
                            }
                        )*
                    }

                    impl ImplDiagnostic for [<$tok Token>] {
                        fn fmt() -> &'static str {
                            $($met)?
                            $($fmt)?
                        }
                    }


                    #[allow(unused_attributes, dead_code)]
                    impl Peek for [<$tok Token>] {
                        fn is(token: &SpannedToken) -> bool {
                            matches!(&token.value, Token::$tok $(
                                ($inner)
                            )?)
                        }
                    }

                    impl Parse for [<$tok Token>] {
                        fn parse(tokens: &mut TokenStream) -> Result<Self, LexingError> {
                            #[allow(unused_parens)]
                            let ($($inner,)* close) = match tokens.next::<Self>() {
                                Some(tok) => (
                                    match tok.value {
                                        Token::$tok $(($inner))? => {
                                            ($($inner, )? ())
                                        },
                                        rest => return Err(LexingError::expected::<[<$tok Token>]>(
                                            rest
                                        ).with_span(
                                            tok.span
                                        ))
                                    }
                                ),
                                None => return Err(LexingError::empty::<[<$tok Token>]>().with_span(
                                        Span::new(tokens.cursor - 1, tokens.cursor)
                                    ))
                            };
                            Ok(Self($($inner,)* close))
                        }
                    }
                )*
            }
        };
    }

    tokens! {
        #[token("::")]
        DoubleColon,
        #[token("{")]
        LBrace,
        #[token("}")]
        RBrace,
        #[token("(")]
        LParen,
        #[token(")")]
        RParen,
        #[token("[")]
        LBracket,
        #[token("]")]
        RBracket,
        #[token(";")]
        Semi,
        #[token(":")]
        Colon,
        #[token(",")]
        Comma,
        #[token("?")]
        QMark,
        #[token("=")]
        Eq,
        #[token("|")]
        Pipe,
        #[token("#")]
        Hash,
        #[token("!")]
        Bang,

        // keywords
        #[token("namespace")]
        KwNamespace,
        #[token("import")]
        KwImport,
        #[token("struct")]
        KwStruct,
        #[token("enum")]
        KwEnum,
        #[token("type")]
        KwType,
        #[token("oneof")]
        KwOneof,
        #[token("bool")]
        KwBool,
        #[token("null")]
        KwNull,
        #[token("str")]
        KwStr,
        #[token("i8")]
        KwI8,
        #[token("i16")]
        KwI16,
        #[token("i32")]
        KwI32,
        #[token("i64")]
        KwI64,
        #[token("u8")]
        KwU8,
        #[token("u16")]
        KwU16,
        #[token("u32")]
        KwU32,
        #[token("u64")]
        KwU64,
        #[token("f16")]
        KwF16,
        #[token("f32")]
        KwF32,
        #[token("f64")]
        KwF64,

        // #[token(" ")]
        // Space,

        // #[token("\t")]
        // Tab,

        // newline preserved (important for single-line comment termination handling if needed later)
        #[regex(r"\r?\n")]
        #[regfmt("\\n")]
        Newline,

        // identifiers
        #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex: &mut Lexer| -> String { lex.slice().to_string() })]
        #[regfmt("identifier")]
        Ident(String),

        // numbers
        #[regex(r"[0-9]+", parse_number)]
        #[regfmt("number")]
        Number(i32),

        #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
        #[regex(r#"'([^'\\]|\\.)*'"#, parse_string)]
        #[regfmt("string")]
        String(String),


        // comments
        #[regex(r"//[^\n]*", |lex: &mut Lexer| -> String { unescape_comment(&lex.slice()[2..]) })]
        #[regfmt("comment")]
        CommentSingleLine(String),

        #[regex(r"/\*([^/*]*)\*/", |lex: &mut Lexer| -> String { unescape_comment(&lex.slice()[2..lex.slice().len()-2]) })]
        #[regfmt("comment")]
        CommentMultiLine(String),


    }

    fn parse_number(lex: &mut logos::Lexer<'_, Token>) -> Result<i32, LexingError> {
        Ok(lex.slice().to_string().parse()?)
    }

    impl fmt::Display for Token {
        fn fmt(
            &self,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            use Token::*;
            match self {
                DoubleColon => write!(f, "::"),
                LBrace => write!(f, "{{"),
                RBrace => write!(f, "}}"),
                LParen => write!(f, "("),
                RParen => write!(f, ")"),
                LBracket => write!(f, "["),
                RBracket => write!(f, "]"),
                Semi => write!(f, ";"),
                Colon => write!(f, ":"),
                Comma => write!(f, ","),
                QMark => write!(f, "?"),
                Eq => write!(f, "="),
                Pipe => write!(f, "|"),
                Hash => write!(f, "#"),
                Bang => write!(f, "!"),
                KwNamespace => write!(f, "namespace"),
                KwImport => write!(f, "import"),
                KwStruct => write!(f, "struct"),
                KwEnum => write!(f, "enum"),
                KwType => write!(f, "type"),
                KwOneof => write!(f, "oneof"),
                KwBool => write!(f, "bool"),
                KwNull => write!(f, "null"),
                KwStr => write!(f, "str"),
                KwI8 => write!(f, "i8"),
                KwI16 => write!(f, "i16"),
                KwI32 => write!(f, "i32"),
                KwI64 => write!(f, "i64"),
                KwU8 => write!(f, "u8"),
                KwU16 => write!(f, "u16"),
                KwU32 => write!(f, "u32"),
                KwU64 => write!(f, "u64"),
                KwF16 => write!(f, "f16"),
                KwF32 => write!(f, "f32"),
                KwF64 => write!(f, "f64"),
                // Space => write!(f, " "),
                // Tab => write!(f, "\t"),
                Newline => write!(f, "\n"),
                Ident(s) => write!(f, "{}", s),
                Number(n) => write!(f, "{}", n),
                String(s) => write!(f, "\"{}\"", s),
                CommentSingleLine(s) => write!(f, "//{}", s),
                CommentMultiLine(s) => write!(f, "/*{}*/", s),
            }
        }
    }

    impl fmt::Debug for Token {
        fn fmt(
            &self,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            match self {
                Self::Ident(s) => write!(f, "Ident({s})"),
                Self::Number(s) => write!(f, "Number({s})"),
                Self::String(s) => write!(f, "String({s})"),
                Self::Newline => write!(f, "\\n"),
                rest => write!(f, "{rest}"),
            }
        }
    }

    fn parse_string(lex: &mut logos::Lexer<Token>) -> Option<String> {
        let slice = lex.slice();
        let inner = &slice[1..slice.len() - 1];
        Some(unescape(inner))
    }

    fn unescape(src: &str) -> String {
        let mut out = String::with_capacity(src.len());
        let mut chars = src.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(n) = chars.next() {
                    match n {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        '\\' => out.push('\\'),
                        '"' => out.push('"'),
                        '\'' => out.push('\''),
                        other => {
                            out.push(other);
                        },
                    }
                }
            } else {
                out.push(c);
            }
        }

        out.trim().to_string()
    }

    fn unescape_comment(src: &str) -> String {
        src.lines()
            .map(|ln| ln.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_start()
            .trim_end()
            .to_string()
    }
}

pub type SpannedToken = Spanned<Token>;

pub mod stream {
    pub use super::*;

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

        pub fn next<T: Parse>(&mut self) -> Option<SpannedToken> {
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
}

pub mod ast {
    use super::*;

    pub trait Peek: Sized {
        fn peek(stream: &TokenStream) -> bool {
            if let Some(token) = stream.peek_unchecked() {
                Self::is(token)
            } else {
                false
            }
        }
        fn is(token: &SpannedToken) -> bool;
    }

    pub trait Parse: Sized {
        fn parse(stream: &mut TokenStream) -> Result<Self, LexingError>;

        fn parse_spanned(stream: &mut TokenStream) -> Result<Spanned<Self>, LexingError> {
            let start = stream.cursor;
            let p = Self::parse(stream)?;
            let end = stream.cursor;
            Ok(Spanned::new(start, end, p))
        }
    }

    impl<T: Peek + Parse> Parse for Spanned<T> {
        fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
            stream.parse()
        }
    }

    impl<T: Peek + Parse> Parse for Option<T> {
        fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
            if stream.peek::<T>() {
                Ok(Some(T::parse(stream)?))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    pub struct RepeatedItem<T: Peek + Parse, Sep: Peek + Parse> {
        pub value: Spanned<T>,
        pub(crate) sep: Option<Spanned<Sep>>,
    }

    #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Repeated<T: Peek + Parse, Sep: Peek + Parse> {
        pub values: Vec<RepeatedItem<T, Sep>>,
    }

    impl<T: Peek + Parse, Sep: Peek + Parse> IntoIterator for Repeated<T, Sep> {
        type Item = RepeatedItem<T, Sep>;
        type IntoIter = <Vec<RepeatedItem<T, Sep>> as IntoIterator>::IntoIter;
        fn into_iter(self) -> Self::IntoIter {
            self.values.into_iter()
        }
    }

    impl<T: Peek + Parse + ImplDiagnostic, Sep: Peek + Parse + Clone + ImplDiagnostic> Parse
        for Repeated<T, Sep>
    {
        fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
            let mut values = Vec::new();

            if !stream.peek::<T>() {
                return Err(LexingError::empty::<T>());
            }

            let first: Spanned<T> = stream.parse()?;
            let mut sep: Option<Spanned<Sep>> = None;

            if stream.peek::<Sep>() {
                let s: Spanned<Sep> = stream.parse()?;
                sep = Some(s);
            }

            values.push(RepeatedItem {
                value: first,
                sep: sep.clone(),
            });

            while let Some(..) = sep {
                if !stream.peek::<T>() {
                    break;
                }

                let next: Spanned<T> = stream.parse()?;
                let mut next_sep: Option<Spanned<Sep>> = None;
                if stream.peek::<Sep>() {
                    let s: Spanned<Sep> = stream.parse()?;
                    next_sep = Some(s);
                }
                values.push(RepeatedItem {
                    value: next,
                    sep: next_sep.clone(),
                });
                sep = next_sep;
            }

            Ok(Self { values })
        }
    }

    impl<T: Parse + Peek> Parse for Vec<Spanned<T>> {
        fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
            let mut out = vec![];
            loop {
                if !stream.peek::<T>() {
                    break;
                }
                out.push(stream.parse()?);
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use serde::Serialize;

    use super::*;

    const SAMPLE_FILES: &[&str] = &[
        include_str!("../samples/enum.pld"),
        include_str!("../samples/ns.pld"),
        include_str!("../samples/array.pld"),
        include_str!("../samples/complex_union.pld"),
        include_str!("../samples/some_import.pld"),
        include_str!("../samples/test_message.pld"),
    ];

    #[test]
    fn samples_lex_without_error_tokens() {
        for (i, src) in SAMPLE_FILES.iter().enumerate() {
            let ts = tokenize(src).unwrap_or_else(|e| panic!("lex errors in sample {src}: {e:#?}"));
            assert!(ts.all().len() > 0);
        }
    }

    #[test_case::test_case(
        "// some inner comment value
        // next line comment
        namespace abc;
        ", |kinds| {
            assert!(matches!(kinds[0], Token::CommentSingleLine(cmt) if cmt == "some inner comment value"));
            assert!(matches!(kinds[1], Token::CommentSingleLine(cmt) if cmt == "next line comment"));
            assert!(matches!(kinds[2], Token::KwNamespace));
            assert!(matches!(kinds[3], Token::Ident(s) if s == "abc"));
        }; "parses single line comment"
    )]
    #[test_case::test_case(
        "/* 
        some
        multiline
        comment
        */
        namespace abc;
        ", |kinds| {
            assert!(matches!(kinds[0], Token::CommentMultiLine(cmt) if cmt == "some\nmultiline\ncomment"));
            assert!(matches!(kinds[1], Token::CommentSingleLine(cmt) if cmt == "next line comment"));
            assert!(matches!(kinds[2], Token::Ident(s) if s == "abc"));
        }; "parses multi line comment"
    )]
    #[test_case::test_case(
        "namespace test;", |kinds| {
            assert!(matches!(kinds[0], Token::KwNamespace));
            assert!(matches!(kinds[1], Token::Ident(s) if s == "test"));
            assert!(matches!(kinds[2], Token::Semi));
        }; "parses namespace"
    )]
    #[test_case::test_case(
        "type cunion_1 = oneof str | i32;", |kinds| {
            assert!(matches!(kinds[0], Token::KwType));
            assert!(matches!(kinds[1], Token::Ident(s) if s == "cunion_1"));
            assert!(matches!(kinds[2], Token::Eq));
            assert!(matches!(kinds[3], Token::KwOneof));
            assert!(matches!(kinds[4], Token::KwStr));
            assert!(matches!(kinds[5], Token::Pipe));
            assert!(matches!(kinds[6], Token::KwI32));
            assert!(matches!(kinds[7], Token::Semi));
        }; "parses oneof type alias"
    )]
    #[test_case::test_case(
        "type aliased = i64;", |kinds| {
            assert!(matches!(kinds[0], Token::KwType));
            assert!(matches!(kinds[1], Token::Ident(s) if s == "aliased"));
            assert!(matches!(kinds[2], Token::Eq));
            assert!(matches!(kinds[3], Token::KwI64));
            assert!(matches!(kinds[4], Token::Semi));
        }; "parses simple type alias"
    )]
    #[test_case::test_case(
        "#[version(1)]", |kinds| {
            assert!(matches!(kinds[0], Token::Hash));
            assert!(matches!(kinds[1], Token::LBracket));
            assert!(matches!(kinds[2], Token::Ident(ver) if ver == "version"));
            assert!(matches!(kinds[3], Token::LParen));
            assert!(matches!(kinds[4], Token::Number(n) if *n == 1));
            assert!(matches!(kinds[5], Token::RParen));
            assert!(matches!(kinds[6], Token::RBracket));
        }; "parses outer meta"
    )]
    #[test_case::test_case(
        "#![version(1)]", |kinds| {
            assert!(matches!(kinds[0], Token::Hash));
            assert!(matches!(kinds[1], Token::Bang));
            assert!(matches!(kinds[2], Token::LBracket));
            assert!(matches!(kinds[3], Token::Ident(ver) if ver == "version"));
            assert!(matches!(kinds[4], Token::LParen));
            assert!(matches!(kinds[5], Token::Number(n) if *n == 1));
            assert!(matches!(kinds[6], Token::RParen));
            assert!(matches!(kinds[7], Token::RBracket));
        }; "parses inner meta"
    )]
    #[test_case::test_case(
        "struct msg_with_union_default {
            a: cunion_1,
            b: i32,
        };", |kinds| {
            assert!(matches!(kinds[0], Token::KwStruct));
            assert!(matches!(kinds[1], Token::Ident(msg) if msg == "msg_with_union_default"));
            assert!(matches!(kinds[2], Token::LBrace));
            assert!(matches!(kinds[3], Token::Ident(a) if a == "a"));
            assert!(matches!(kinds[4], Token::Colon));
            assert!(matches!(kinds[5], Token::Ident(t) if t == "cunion_1"));
            assert!(matches!(kinds[6], Token::Comma));
            assert!(matches!(kinds[7], Token::Ident(b) if b == "b"));
            assert!(matches!(kinds[8], Token::Colon));
            assert!(matches!(kinds[9], Token::KwI32));
            assert!(matches!(kinds[10], Token::Comma));
            assert!(matches!(kinds[11], Token::RBrace));
        }; "parses struct"
    )]
    fn keyword_then_ident<F: Fn(Vec<&Token>)>(
        src: &str,
        expect: F,
    ) {
        let ts = tokenize(src).unwrap();
        let kinds: Vec<&Token> = ts
            .all()
            .iter()
            .map(|t| &t.value)
            // .filter(|it| !matches!(it, Token::Newline))
            .collect();
        println!("{kinds:#?}");
        (expect)(kinds);
    }

    #[test]
    fn test_increment_parse() -> Result<(), LexingError> {
        let src = "namespace abc;";

        let mut ts = tokenize(src).unwrap();

        let _: Spanned<KwNamespaceToken> = ts.parse()?;
        let _: Spanned<IdentToken> = ts.parse()?;

        Ok(())
    }

    #[test_case::test_case(
        "namespace i32",
        |mut ts: TokenStream| {
            let _: Spanned<KwNamespaceToken> = ts.parse().unwrap();
            ts.parse::<IdentToken>().unwrap_err()
        }, "expected identifier, found 'i32'", Span::new(10, 13);
        "expectation error: namespace followed by keyword"
    )]
    fn test_incremental_and_error_cases(
        src: &str,
        expect: impl FnOnce(TokenStream) -> LexingError,
        err_message: &str,
        expect_span: Span,
    ) {
        let ts = tokenize(src).unwrap();
        let err = expect(ts);
        if let LexingError::Spanned { span, source } = err {
            assert_eq!(format!("{source}"), err_message);

            assert_eq!(
                span.start, expect_span.start,
                "start of error span should align"
            );
            assert_eq!(span.end, expect_span.end, "end of error span should align");
        } else {
            panic!("expected spanned error");
        }
    }

    #[test_case::test_case(
        "// multiple
        // single
        // line
        // comments
        ", 
        serde_json::json!({"span":{"end":7,"start":0},"value":[{"span":{"end":1,"start":0},"value":["multiple"]},{"span":{"end":3,"start":1},"value":["single"]},{"span":{"end":5,"start":3},"value":["line"]},{"span":{"end":7,"start":5},"value":["comments"]}]}),
        PhantomData::<tokens::CommentSingleLineToken>;
        "parses single line comments over new lines"
    )]
    #[test_case::test_case(
        "/*
            the inner comment value
        */ /*
            next comment value
        */", 
        serde_json::json!({"span":{"end":2,"start":0},"value":[{"span":{"end":1,"start":0},"value":["the inner comment value"]},{"span":{"end":2,"start":1},"value":["next comment value"]}]}),
        PhantomData::<tokens::CommentMultiLineToken>;
        "parses comments over new lines"
    )]
    fn test_to_vec<T>(
        src: &str,
        expect: serde_json::Value,
        _: PhantomData<T>,
    ) -> Result<(), LexingError>
    where
        T: Peek + Parse + Serialize + ImplDiagnostic + Clone, {
        let mut ts = tokenize(src).unwrap();
        let found: Spanned<Vec<Spanned<T>>> = ts.parse()?;

        let as_j = serde_json::to_value(&found).unwrap();
        let debug = serde_json::to_string(&as_j).unwrap();

        println!("found: {debug}");

        assert_eq!(as_j, expect, "spans and span values must be exactly equal");
        Ok(())
    }

    #[test_case::test_case(
        "
            a,
            b,
            c
        ", 
        serde_json::json!({"span":{"end":44,"start":0},"value":{"values":[{"sep":{"span":{"end":15,"start":14},"value":null},"value":{"span":{"end":14,"start":0},"value":["a"]}},{"sep":{"span":{"end":30,"start":29},"value":null},"value":{"span":{"end":29,"start":15},"value":["b"]}},{"sep":null,"value":{"span":{"end":44,"start":30},"value":["c"]}}]}}), 
        PhantomData::<(tokens::IdentToken, tokens::CommaToken)>;
        "repeated idents with new lines"
    )]
    fn test_repeated<T, Sep>(
        src: &str,
        expect: serde_json::Value,
        _: PhantomData<(T, Sep)>,
    ) -> Result<(), LexingError>
    where
        T: Peek + Parse + Serialize + ImplDiagnostic + Clone,
        Sep: Peek + Parse + Serialize + ImplDiagnostic + Clone, {
        let mut ts = tokenize(src).unwrap();
        let found: Spanned<Repeated<T, Sep>> = ts.parse()?;
        let as_j = serde_json::to_value(&found).unwrap();
        let debug = serde_json::to_string(&as_j).unwrap();

        println!("found: {debug}");

        assert_eq!(as_j, expect, "spans and span values must be exactly equal");
        Ok(())
    }
}
