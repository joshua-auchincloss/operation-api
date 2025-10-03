#![allow(
    clippy::large_enum_variant,
    clippy::result_large_err,
    clippy::never_loop,
    clippy::ptr_arg
)]

pub mod ast;
pub mod ctx;
pub mod defs;
pub mod diagnostics;
pub mod tokens;
pub(crate) mod utils;

#[cfg(test)]
pub(crate) mod tst;

use std::convert::Infallible;

use crate::{defs::Ident, tokens::LexingError};
use thiserror::Error;

pub use tokens::{ImplDiagnostic, Parse, Peek};

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{namespace:#?} has conflicts. {ident} is declared multiple times.")]
    IdentConflict { namespace: Vec<Ident>, ident: Ident },

    #[error("namespace is not declared")]
    NsNotDeclared,

    #[error("only one namespace may be declared in a payload declaration file.")]
    NsConflict,

    #[error("resolution error. could not resolve {ident}")]
    ResolutionError { ident: Ident },

    #[error("{inner}")]
    WithSpan {
        #[source]
        inner: Box<Error>,
        start: usize,
        end: usize,
    },

    #[error("version attribute conflict: values={values:?}")]
    VersionConflict {
        values: Vec<usize>,
        spans: Vec<(usize, usize)>,
    },

    #[error("lex error: invalid character '{ch}'")]
    LexError { ch: char, start: usize, end: usize },

    #[error("{0}")]
    AstError(#[from] tokens::LexingError),
}

impl Error {
    pub fn conflict(
        namespace: Vec<Ident>,
        ident: Ident,
    ) -> Self {
        Self::IdentConflict { namespace, ident }
    }
    pub fn conflict_spanned<Ns: Into<Vec<defs::Ident>>, Id: Into<defs::Ident>>(
        ns: Ns,
        id: Id,
        start: usize,
        end: usize,
    ) -> Self {
        Self::IdentConflict {
            namespace: ns.into(),
            ident: id.into(),
        }
        .with_span(start, end)
    }

    pub fn resolution(ident: Ident) -> Self {
        Self::ResolutionError { ident }
    }
    pub fn resolution_spanned(
        id: defs::Ident,
        start: usize,
        end: usize,
    ) -> Self {
        Self::ResolutionError { ident: id }.with_span(start, end)
    }

    pub fn with_span(
        self,
        start: usize,
        end: usize,
    ) -> Self {
        Error::WithSpan {
            inner: Box::new(self),
            start,
            end,
        }
    }

    pub fn then_with_span(
        start: usize,
        end: usize,
    ) -> impl FnOnce(Self) -> Self {
        move |this: Error| this.with_span(start, end)
    }

    pub fn to_report_with(
        &self,
        path: &std::path::Path,
        source: &str,
        override_span: Option<(usize, usize)>,
    ) -> miette::Report {
        use miette::NamedSource;
        let named = NamedSource::new(path.to_string_lossy(), source.to_string());

        let effective = match (override_span, self) {
            (Some(s), _) => Some(s),
            (None, Error::AstError(inner)) => {
                match inner {
                    LexingError::Spanned { span, .. } => Some((span.start, span.end)),
                    _ => None,
                }
            },
            (None, Error::WithSpan { start, end, .. }) => Some((*start, *end)),
            (None, Error::VersionConflict { spans, .. }) if !spans.is_empty() => Some(spans[0]),
            _ => None,
        };

        if let Some((s, e)) = effective {
            let diag = crate::diagnostics::SpanDiagnostic::new(
                &crate::defs::Spanned::new(s, e, ()),
                path,
                source,
                format!("{self}"),
                format!("{self}"),
                None,
            );
            miette::Report::new(diag)
        } else {
            miette::Report::msg(format!("{self}")).with_source_code(named)
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
