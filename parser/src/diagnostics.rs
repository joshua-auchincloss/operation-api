use crate::defs::Spanned;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct SpanDiagnostic {
    #[source_code]
    src: NamedSource<String>,

    #[label]
    span: Option<SourceSpan>,

    message: String,
    #[allow(dead_code)]
    label: String,
    #[diagnostic(help)]
    help: Option<String>,
}

impl SpanDiagnostic {
    pub fn new<T>(
        sp: &Spanned<T>,
        path: &Path,
        source: &str,
        message: impl Into<String>,
        label: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self {
            src: NamedSource::new(path.to_string_lossy(), source.to_string()),
            span: Some(SourceSpan::new(sp.start.into(), sp.end - sp.start)),
            message: message.into(),
            label: label.into(),
            help,
        }
    }

    pub fn no_span(
        path: &Path,
        source: &str,
        message: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self {
            src: NamedSource::new(path.to_string_lossy(), source.to_string()),
            span: None,
            message: message.into(),
            label: String::new(),
            help,
        }
    }
}

pub trait SpannedExt<T> {
    fn error(
        &self,
        path: &Path,
        source: &str,
        message: impl Into<String>,
    ) -> SpanDiagnostic;
}

impl<T> SpannedExt<T> for Spanned<T> {
    fn error(
        &self,
        path: &Path,
        source: &str,
        message: impl Into<String>,
    ) -> SpanDiagnostic {
        SpanDiagnostic::new(self, path, source, message, "here", None)
    }
}
