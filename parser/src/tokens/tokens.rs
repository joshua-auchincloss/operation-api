use crate::{
    defs::{Spanned, span::Span},
    tokens::{ImplDiagnostic, error::LexingError},
};
use logos::Logos;
use std::fmt;

pub type SpannedToken = Spanned<Token>;

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
                #[doc = concat!(
                    "Represents `", $($met)? $($fmt)?, "` token(s)"
                )]
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
                impl crate::tokens::ast::Peek for [<$tok Token>] {
                    fn is(token: &SpannedToken) -> bool {
                        matches!(&token.value, Token::$tok $(
                            ($inner)
                        )?)
                    }
                }

                impl crate::tokens::ast::Parse for [<$tok Token>] {
                    fn parse(tokens: &mut crate::tokens::stream::TokenStream) -> Result<Self, LexingError> {
                        #[allow(unused_parens)]
                        let ($($inner,)* close) = match tokens.next() {
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

pub(crate) use tokens as declare_tokens;

declare_tokens! {
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
    #[token("error")]
    KwError,
    #[token("operation")]
    KwOperation,


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

    #[regex(r"\r?\n")]
    #[regfmt("\\n")]
    Newline,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex: &mut logos::Lexer<'_, Token>| -> String { lex.slice().to_string() })]
    #[regfmt("identifier")]
    Ident(String),

    #[regex(r"[0-9]+", parse_number)]
    #[regfmt("number")]
    Number(i32),

    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    #[regex(r#"'([^'\\]|\\.)*'"#, parse_string)]
    #[regfmt("string")]
    String(String),

    #[regex(r"//[^\n]*", |lex: &mut logos::Lexer<'_, Token>| -> String { unescape_comment(&lex.slice()[2..]) })]
    #[regfmt("comment")]
    CommentSingleLine(String),

    #[regex(r"/\*([^/*]*)\*/", |lex: &mut logos::Lexer<'_, Token>| -> String { unescape_comment(&lex.slice()[2..lex.slice().len()-2]) })]
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
            KwError => write!(f, "error"),
            KwOperation => write!(f, "operation"),
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

fn parse_string(lex: &mut logos::Lexer<'_, Token>) -> Option<String> {
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

#[macro_export]
macro_rules! Token {
    [::] => { $crate::tokens::tokens::DoubleColonToken };
    [;] => { $crate::tokens::tokens::SemiToken };
    [:] => { $crate::tokens::tokens::ColonToken };
    [,] => { $crate::tokens::tokens::CommaToken };
    [?] => { $crate::tokens::tokens::QMarkToken };
    [=] => { $crate::tokens::tokens::EqToken };
    [|] => { $crate::tokens::tokens::PipeToken };
    [#] => { $crate::tokens::tokens::HashToken };
    [!] => { $crate::tokens::tokens::BangToken };
    [namespace] => { $crate::tokens::tokens::KwNamespaceToken };
    [import] => { $crate::tokens::tokens::KwImportToken };
    [struct] => { $crate::tokens::tokens::KwStructToken };
    [enum] => { $crate::tokens::tokens::KwEnumToken };
    [type] => { $crate::tokens::tokens::KwTypeToken };
    [oneof] => { $crate::tokens::tokens::KwOneofToken };
    [error] => { $crate::tokens::tokens::KwErrorToken };
    [operation] => { $crate::tokens::tokens::KwOperationToken };
    [bool] => { $crate::tokens::tokens::KwBoolToken };
    [null] => { $crate::tokens::tokens::KwNullToken };
    [str] => { $crate::tokens::tokens::KwStrToken };
    [i8] => { $crate::tokens::tokens::KwI8Token };
    [i16] => { $crate::tokens::tokens::KwI16Token };
    [i32] => { $crate::tokens::tokens::KwI32Token };
    [i64] => { $crate::tokens::tokens::KwI64Token };
    [u8] => { $crate::tokens::tokens::KwU8Token };
    [u16] => { $crate::tokens::tokens::KwU16Token };
    [u32] => { $crate::tokens::tokens::KwU32Token };
    [u64] => { $crate::tokens::tokens::KwU64Token };
    [f16] => { $crate::tokens::tokens::KwF16Token };
    [f32] => { $crate::tokens::tokens::KwF32Token };
    [f64] => { $crate::tokens::tokens::KwF64Token };
    [newline] => { $crate::tokens::tokens::NewlineToken };
    [ident] => { $crate::tokens::tokens::IdentToken };
    [number] => { $crate::tokens::tokens::NumberToken };
    [string] => { $crate::tokens::tokens::StringToken };
    [comment_single_line] => { $crate::tokens::tokens::CommentSingleLineToken };
    [comment_multi_line] => { $crate::tokens::tokens::CommentMultiLineToken };
}

#[macro_export]
macro_rules! SpannedToken {
    ($met: tt) => {
        $crate::defs::Spanned::<$crate::Token![$met]>
    };
}
