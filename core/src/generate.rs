pub mod context;
pub mod files;
pub mod matcher;
pub mod python;
pub mod remote;
pub mod rust;

use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use config::File;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;

use crate::{
    Enum, Error, Operation, Result, Struct,
    context::Context,
    generate::{
        context::WithNsContext,
        files::{MemFlush, WithFlush},
        remote::RemoteConfig,
        rust::RustGenerator,
    },
};

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
    pub output_dir: PathBuf,
    pub opts: Ext,

    #[serde(default)]
    pub mem: bool,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RustConfig {}

impl ConfigExt for RustConfig {}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct JsonSchemaConfig {}

impl ConfigExt for JsonSchemaConfig {}

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
    pub sources: Source,

    #[serde(default = "default_targets")]
    pub targets: Vec<Target>,

    #[serde(default)]
    pub languages: Vec<Language>,

    #[serde(default)]
    pub rust: Option<GenOpts<RustConfig>>,
}

impl GenerationConfig {
    pub fn new<'a, S: Into<&'a str>>(dir: Option<S>) -> Result<Self> {
        let file_name = format!(
            "{}",
            PathBuf::from(dir.map(|s| s.into()).unwrap_or("./"))
                .join("op-gen")
                .display()
        );
        Ok(
            config::ConfigBuilder::<config::builder::DefaultState>::default()
                .add_source(File::with_name(&file_name).required(true))
                .add_source(config::Environment::default().prefix("OP"))
                .build()
                .map_err(Error::from_with_source_init(file_name.clone()))?
                .try_deserialize()
                .map_err(Error::from_with_source_init(file_name.clone()))?,
        )
    }
}

pub trait Generate<State: Sync + Send, Ext: ConfigExt + Sync + Send>
where
    Self: LanguageTrait + Sized + Sync + Send, {
    fn on_create<'s>(
        state: &WithNsContext<'s, State, Ext, Self>,
        fname: &PathBuf,
        f: &mut Box<dyn WithFlush>,
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
        mem_flush: Option<MemFlush>,
    ) -> Result<()> {
        let state = Arc::new(self.new_state(opts));
        let mut ctx_ns = BTreeMap::new();

        for ns in ctx.namespaces.values() {
            let ns_ctx = WithNsContext::new(
                ns,
                state.clone(),
                opts,
                Box::new(|state, fname, f| Self::on_create(state, fname, f)),
                mem_flush.clone(),
            );

            for def in ns.defs.values() {
                self.gen_struct(&ns_ctx, def)?;
            }

            for op in ns.ops.values() {
                self.gen_operation(&ns_ctx, op)?;
            }

            for enm in ns.enums.values() {
                self.gen_enum(&ns_ctx, enm)?;
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

    fn gen_enum<'s>(
        &self,
        state: &WithNsContext<'s, State, Ext, Self>,
        def: &Enum,
    ) -> Result<()>;
}

impl GenerationConfig {
    pub fn sources(&self) -> Result<Vec<PathBuf>> {
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

    pub fn set_mem(
        &mut self,
        mem: bool,
    ) {
        if let Some(rs) = &mut self.rust {
            rs.mem = mem;
        }
    }
}

pub struct Generation {
    pub config: GenerationConfig,
    pub ctx: Context,
}

impl Generation {
    pub fn new(config: GenerationConfig) -> Result<Self> {
        let ctx = config.get_ctx()?;
        Ok(Self { config, ctx })
    }

    pub fn generate_all_sync(
        &self,
        mem_flush: Option<MemFlush>,
    ) -> crate::Result<()> {
        let mut generators = vec![];

        if let Some(rust) = &self.config.rust {
            generators.push(Box::new(|| {
                RustGenerator.gen_ctx(&self.ctx, rust, mem_flush.clone())
            }));
        }

        for it in generators
            .into_par_iter()
            .map(|handle| (*handle)())
            .collect::<Vec<_>>()
        {
            it?;
        }

        Ok(())
    }

    pub async fn generate_all(
        &self,
        mem_flush: Option<MemFlush>,
    ) -> crate::Result<()> {
        let mut futs = vec![];
        if let Some(rust) = &self.config.rust {
            let generator = RustGenerator;
            futs.push(Box::pin(async move {
                generator.gen_ctx(&self.ctx, rust, mem_flush.clone())
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
    use crate::{Definitions, generate::files::MemCollector};

    use super::*;

    #[test]
    fn test_gen_mem() -> crate::Result<()> {
        let mut conf = GenerationConfig::new(Some("../samples/config-a")).unwrap();
        conf.set_mem(true);

        let collector = MemCollector::new();

        let generate = Generation::new(conf)?;
        generate.generate_all_sync(Some(collector.mem_flush()))?;

        let state = collector.files();
        assert_eq!(state.keys().len(), 2);

        Ok(())
    }

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
                        "../samples/basic-enum.toml".into(),
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
                PathBuf::from("../samples/basic-enum.toml"),
                PathBuf::from("../samples/basic-struct.toml"),
                PathBuf::from("../samples/test-struct-readme.toml"),
                PathBuf::from("../samples/test-struct-text.toml"),
                PathBuf::from("../samples/test-struct-with-enum.toml"),
            ]
        }

        let ctx = conf.get_ctx().unwrap();
        let ns = ctx
            .namespaces
            .get(&"abc.corp.namespace".into())
            .unwrap();

        Definitions::NamespaceV1(ns.clone())
            .export("../samples/abc_corp_namespace.toml".into())
            .unwrap();

        insta::assert_yaml_snapshot!(ctx.namespaces);

        let generator = Generation::new(conf).unwrap();
        generator.generate_all(None).await.unwrap();
    }
}
