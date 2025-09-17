pub mod matcher;
pub mod remote;
pub mod rust;

use std::{fmt::Debug, path::PathBuf};

use config::File;
use serde::Deserialize;

use crate::{
    Definitions, Operation, Struct,
    generate::{remote::RemoteConfig, rust::RustGenerator},
};

#[derive(thiserror::Error, Debug)]
pub enum GenerationError {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("glob error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
}

type Result<T, E = GenerationError> = std::result::Result<T, E>;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Client,
    Server,
    Types,
}

pub trait ConfigExt: Debug + PartialEq + Clone {}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GenOpts<Ext: ConfigExt> {
    output_dir: PathBuf,
    opts: Ext,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RustConfig {}

impl ConfigExt for RustConfig {}

crate::default!(
    Vec<Target>: { targets = vec![Target::Types] },
);

#[derive(Deserialize, PartialEq, Debug)]
pub struct Source {
    #[serde(default)]
    remote: Vec<RemoteConfig>,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GenerationConfig {
    sources: Source,

    #[serde(default = "default_targets")]
    targets: Vec<Target>,

    #[serde(default)]
    languages: Vec<Language>,

    #[serde(default)]
    rust: Option<GenOpts<RustConfig>>,
}

impl GenerationConfig {
    pub fn new<'a, S: Into<&'a str>>(dir: Option<S>) -> Result<Self> {
        Ok(
            config::ConfigBuilder::<config::builder::DefaultState>::default()
                .add_source(
                    File::with_name(&format!(
                        "{}",
                        PathBuf::from(dir.map(|s| s.into()).unwrap_or("./"))
                            .join("op-gen")
                            .display()
                    ))
                    .required(true),
                )
                .add_source(config::Environment::default().prefix("OP"))
                .build()?
                .try_deserialize()?,
        )
    }
}

pub trait Generate<Ext: ConfigExt> {
    fn gen_operation(
        &self,
        def: Operation,
        opts: GenOpts<Ext>,
    ) -> Result<()>;
    fn gen_struct(
        &self,
        def: Struct,
        opts: GenOpts<Ext>,
    ) -> Result<()>;
    fn gen_definition(
        &self,
        def: Definitions,
        opts: GenOpts<Ext>,
    ) -> Result<()> {
        match def {
            Definitions::OperationV1(def) => self.gen_operation(def, opts),
            Definitions::StructV1(def) => self.gen_struct(def, opts),
            // we can skip these as we dont have/use field aliases in generated code
            Definitions::FieldV1(..) => Ok(()),
        }
    }
}

impl GenerationConfig {
    fn sources(&self) -> Result<Vec<PathBuf>> {
        matcher::walk(&self.sources)
    }

    pub async fn generate_all(&self) -> Result<()> {
        let files = self.sources()?;

        let mut futs = vec![];
        if let Some(rust) = &self.rust {
            let generator = RustGenerator;
            futs.push(Box::pin(async move {
                // generator.gen_definition(def, rust.clone())
                Ok::<_, GenerationError>(())
            }));
        }

        for fut in futs {
            let _ = fut.await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_loader() {
        let conf = GenerationConfig::new(Some("../samples/config-a")).unwrap();

        assert_eq! {
            conf, GenerationConfig {
                targets: vec![Target::Client, Target::Server, Target::Types],
                languages: vec![Language::Rust],
                sources: Source {
                    remote: vec![
                        RemoteConfig {
                            url: "http://localhost:9009/dynamic.toml".into(),
                            headers: Default::default()
                        }
                    ],
                    include: vec![
                        "../samples/basic-op.toml".into(),
                        "../samples/test-struct*.toml".into()
                    ],
                    exclude: vec![
                        "*basic-op*".into()
                    ]
                },
                rust: Some(GenOpts{
                    output_dir: "../samples/gen-a".into(),
                    opts: RustConfig {  }
                })
            }
        }

        assert_eq! {
            conf.sources().unwrap(),
            vec![
                PathBuf::from("../samples/test-struct-readme.toml"),
                PathBuf::from("../samples/test-struct-text.toml"),
            ]
        }
    }
}
