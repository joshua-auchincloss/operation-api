use std::path::{Path, PathBuf};

use crate::defs::Payload;
use miette::{Context, IntoDiagnostic};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/ident.pest"]
pub struct PayloadParser;

impl PayloadParser {
    pub fn parse_data<P: Into<PathBuf>>(
        source: P,
        s: &str,
    ) -> miette::Result<Payload> {
        tracing::debug!("{s}");
        let source = source.into();
        let parsed = Self::parse(Rule::payloads, s)
            .map_err(crate::Error::from)
            .map_err(|err| err.to_report_with(&source, s, None))?;

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
