pub mod context;
pub mod files;
pub mod matcher;
pub mod remote;
pub mod rust;

use std::{
    collections::BTreeMap,
    fmt::Debug,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use config::File;
use serde::Deserialize;

use crate::{
    Operation, Struct,
    context::Context,
    generate::{context::WithNsContext, remote::RemoteConfig, rust::RustGenerator},
};

#[derive(thiserror::Error, Debug)]
pub enum GenerationError {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("glob error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

type Result<T, E = GenerationError> = std::result::Result<T, E>;

pub trait LanguageTrait {
    fn file_case() -> convert_case::Case<'static>;
    fn file_ext() -> &'static str;

    fn file_name<P: AsRef<Path>>(name: P) -> PathBuf {
        PathBuf::from(format!("{}.{}", name.as_ref().display(), Self::file_ext()))
    }
}

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

    #[serde(default = "crate::utils::default_no")]
    mem: bool,
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

pub trait Generate<State, Ext: ConfigExt>
where
    Self: LanguageTrait + Sized, {
    fn on_create<'s>(
        state: &WithNsContext<'s, State, Ext, Self>,
        fname: &PathBuf,
        f: &mut Box<dyn Write>,
    ) -> std::io::Result<()>;

    fn new_state<'ns>(
        &self,
        opts: &'ns GenOpts<Ext>,
    ) -> State;

    #[allow(unused)]
    fn with_all_namespaces<'ns>(
        &self,
        ctx: &Context,
        opts: &'ns GenOpts<Ext>,
        ctx_ns: BTreeMap<crate::Ident, WithNsContext<'ns, State, Ext, Self>>,
    ) -> Result<()> {
        Ok(())
    }

    fn gen_ctx<'ns>(
        &self,
        ctx: &Context,
        opts: &'ns GenOpts<Ext>,
    ) -> Result<()> {
        let state = Arc::new(self.new_state(opts));
        let mut ctx_ns = BTreeMap::new();
        for ns in ctx.namespaces.values() {
            let ns_ctx = WithNsContext::new(
                ns,
                state.clone(),
                opts,
                Box::new(|state, fname, f| Self::on_create(state, fname, f)),
            );

            for def in ns.defs.values() {
                self.gen_struct(&ns_ctx, def)?;
            }

            for op in ns.ops.values() {
                self.gen_operation(&ns_ctx, op)?;
            }

            ctx_ns.insert(ns.name.clone(), ns_ctx);
        }

        self.with_all_namespaces(ctx, opts, ctx_ns)?;

        Ok(())
    }

    fn gen_struct<'s>(
        &self,
        state: &WithNsContext<'s, State, Ext, Self>,
        def: &Struct,
    ) -> Result<()>;

    fn gen_operation<'s>(
        &self,
        state: &WithNsContext<'s, State, Ext, Self>,
        def: &Operation,
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
                    opts: RustConfig {  },
                    mem: false
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

        insta::assert_yaml_snapshot!(ctx.namespaces);

        conf.generate_all().await.unwrap();
    }
}
