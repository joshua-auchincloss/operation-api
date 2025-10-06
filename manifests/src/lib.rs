use std::path::{Path, PathBuf};

pub mod config;
pub mod package;
pub mod rules;
pub mod version;

pub use crate::config::NewForConfig;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[{file}] {err}")]
    WithSource { file: PathBuf, err: Box<Self> },
    #[error("config error: {0}")]
    ConfigError(#[from] ::config::ConfigError),
    #[error("validation error: {0}")]
    ValidationError(#[from] ::validator::ValidationError),
    #[error("validation errors: {0}")]
    ValidationErrors(#[from] ::validator::ValidationErrors),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    pub fn with_source(
        self,
        file: impl AsRef<Path>,
    ) -> Self {
        Self::WithSource {
            file: file.as_ref().to_path_buf(),
            err: Box::new(self),
        }
    }

    pub fn from_with_source_init<E: Into<Error>>(file: impl AsRef<Path>) -> impl FnOnce(E) -> Self {
        |err| err.into().with_source(file)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
