pub mod matcher;
pub mod remote;
pub mod rust;

use std::{fmt::Debug, path::PathBuf};

use config::File;
use serde::Deserialize;

use crate::{
    Definitions, Operation, Struct,
    context::Context,
    generate::{remote::RemoteConfig, rust::RustGenerator},
    namespace::WithNsContext,
};

#[derive(thiserror::Error, Debug)]
pub enum GenerationError {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("glob error: {0}")]
    Glob(#[from] glob::GlobError),
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

pub trait Generate<State, Ext: ConfigExt> {
    fn new_state<'ns>(
        &self,
        opts: &GenOpts<Ext>,
    ) -> State;

    fn gen_ctx<'ns>(
        &self,
        ctx: &'ns Context,
        opts: &GenOpts<Ext>,
    ) -> Result<()> {
        let mut state = self.new_state(opts);

        for ns in ctx.namespaces.values() {
            let mut ns_ctx = WithNsContext::new(ns, &mut state);

            for def in ns.defs.values() {
                self.gen_struct(&mut ns_ctx, def)?;
            }

            for op in ns.ops.values() {
                self.gen_operation(&mut ns_ctx, op)?;
            }
        }

        Ok(())
    }

    fn gen_operation<'ns>(
        &self,
        state: &mut WithNsContext<'ns, State>,
        def: &Operation,
    ) -> Result<()>;

    fn gen_struct<'ns>(
        &self,
        state: &mut WithNsContext<'ns, State>,
        def: &Struct,
    ) -> Result<()>;
}

impl GenerationConfig {
    fn sources(&self) -> Result<Vec<PathBuf>> {
        matcher::walk(&self.sources)
    }

    pub fn get_ctx(&self) -> crate::Result<Context> {
        let mut ctx = Context::new();

        for source in self.sources()? {
            ctx.load_from_source(source)?;
        }

        ctx.finish()?;

        Ok(ctx)
    }

    pub async fn generate_all(&self) -> crate::Result<()> {
        let ctx = self.get_ctx()?;

        let mut futs = vec![];
        if let Some(rust) = &self.rust {
            let generator = RustGenerator;
            futs.push(Box::pin(async move { generator.gen_ctx(&ctx, rust) }));
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

    #[tokio::test]
    async fn test_config_loader() {
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
                        "../samples/basic-struct.toml".into(),
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
                PathBuf::from("../samples/basic-struct.toml"),
                PathBuf::from("../samples/test-struct-readme.toml"),
                PathBuf::from("../samples/test-struct-text.toml"),
            ]
        }

        let ctx = conf.get_ctx().unwrap();

        insta::assert_debug_snapshot!(ctx.namespaces);

        conf.generate_all().await.unwrap();
    }
}
