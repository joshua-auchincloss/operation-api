use std::path::{Path, PathBuf};

use crate::{
    Result,
    defs::{FromInner, Payload},
};
use miette::{
    Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, SourceOffset, SourceSpan,
};
use pest::{Parser, iterators::Pairs};
use pest_derive::Parser;

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("parsing error")]
struct SourceError {
    #[source_code]
    src: NamedSource<String>,

    #[label("here")]
    bad_bit: SourceSpan,

    #[diagnostic(help)]
    help: String,
}

#[derive(Parser)]
#[grammar = "../grammar/ident.pest"]
pub struct PayloadParser;

fn labelled_span(error: &pest::error::Error<Rule>) -> SourceSpan {
    match error.location {
        pest::error::InputLocation::Pos(loc) => SourceSpan::new(loc.into(), 1),
        pest::error::InputLocation::Span((start, end)) => {
            SourceSpan::new(start.into(), (end - start).into())
        },
    }
}

impl PayloadParser {
    pub fn parse_data<'p, P: Into<PathBuf>>(
        source: P,
        s: &'p str,
    ) -> miette::Result<Payload> {
        tracing::debug!("{s}");
        let source = source.into();
        let parsed = Self::parse(Rule::payloads, s.as_ref()).map_err(|e| {
            let src = NamedSource::new(source.clone().to_string_lossy().to_string(), s.to_string());

            SourceError {
                src,
                bad_bit: labelled_span(&e),
                help: e.variant.message().into(),
            }
        })?;

        // #[cfg(all(debug_assertions, not(feature = "bench")))]
        std::fs::write("parsed.out", format!("{parsed:#?}")).unwrap();
        std::fs::write("parsed.json", parsed.to_json()).unwrap();

        crate::defs::Payload::build(source, parsed)
            .into_diagnostic()
            .wrap_err("failed to parse ast")
    }

    pub fn parse_file<P: AsRef<Path>>(f: P) -> miette::Result<Payload> {
        let data = std::fs::read_to_string(&f)
            .into_diagnostic()
            .wrap_err("failed to read source file")?;
        Self::parse_data(f.as_ref().to_path_buf(), &data)
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_matches_debug, tst::logging};

    use super::*;

    #[test]
    fn test_parses_complex_message() {
        logging();

        let p = PayloadParser::parse_file("samples/test_message.pld").unwrap();
        assert_matches_debug!("../samples/test_message_parse.txt", p);
    }

    #[test]
    fn test_parses_enums() {
        logging();

        let p = PayloadParser::parse_file("samples/enum.pld").unwrap();
        assert_matches_debug!("../samples/enum_parse.txt", p);
    }

    #[test]
    fn test_namespace() {
        logging();

        let p = PayloadParser::parse_file("samples/ns.pld").unwrap();
        assert_matches_debug!("../samples/ns_parse.txt", p);
    }

    #[test]
    fn test_message_with_enum() {
        logging();

        let p = PayloadParser::parse_file("samples/message_with_enum.pld").unwrap();
        assert_matches_debug!("../samples/message_with_enum_parse.txt", p);
    }

    #[test]
    fn test_parses_arrays() {
        logging();

        let p = PayloadParser::parse_file("samples/array.pld").unwrap();
        assert_matches_debug!("../samples/array_parse.txt", p);
    }

    #[test]
    fn test_parses_complex_unions() {
        logging();

        let p = PayloadParser::parse_file("samples/complex_union.pld").unwrap();
        assert_matches_debug!("../samples/complex_union_parse.txt", p);
    }
}
