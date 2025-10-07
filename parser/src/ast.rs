pub mod anonymous;
pub mod array;
pub mod comment;
pub mod enm;
pub mod err;
pub mod import;
pub mod items;
pub mod meta;
pub mod namespace;
pub mod one_of;
pub mod op;
pub mod path;
pub mod strct;
pub mod ty;
pub mod ty_def;
pub mod union;
pub mod variadic;

use std::path::Path;

use miette::IntoDiagnostic;

use crate::{
    Parse,
    defs::Spanned,
    tokens::{self, AstResult, tokenize},
};

pub struct AstStream {
    nodes: Vec<Spanned<items::Items>>,
}

impl crate::Parse for AstStream {
    fn parse(stream: &mut crate::tokens::TokenStream) -> AstResult<Self> {
        Ok(Self {
            nodes: Vec::parse(stream)?,
        })
    }
}

impl AstStream {
    pub fn from_string(src: &str) -> AstResult<Self> {
        let mut tt = tokenize(src)?;
        Self::parse(&mut tt)
    }

    pub fn from_file(path: impl AsRef<Path>) -> miette::Result<Self> {
        let data = std::fs::read_to_string(path.as_ref()).into_diagnostic()?;

        Self::from_string(&data).map_err(|lex| {
            let crate_err: crate::Error = lex.into();
            crate_err.to_report_with(path.as_ref(), &data, None)
        })
    }
}

impl IntoIterator for AstStream {
    type Item = Spanned<items::Items>;
    type IntoIter = <Vec<Spanned<items::Items>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl crate::tokens::ToTokens for AstStream {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        for node in &self.nodes {
            tt.write(node);
            tt.write(&tokens::NewlineToken::new());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tokens::ToTokens;

    use super::*;

    #[test_case::test_case("samples/array.pld")]
    fn round_trip(path: &str) {
        let data = std::fs::read_to_string(path).unwrap();
        let ast = AstStream::from_file(path).unwrap();

        let out = ast.tokens();
        let fmt = format!("{out}");
        assert_eq!(data, fmt, "expected:\n{data}\ngot:\n{fmt}");
    }
}
