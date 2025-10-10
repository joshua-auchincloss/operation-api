use std::path::PathBuf;

use operation_api_manifests::NewForConfig;

#[derive(Default, clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Debug,
    Trace,
    #[default]
    Info,
    Error,
    Warn,
}

impl From<LogLevel> for tracing::Level {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
        }
    }
}

#[derive(clap::Parser, Debug, Clone)]
#[clap(name = "")]
pub struct Cli {
    #[clap(
        long,
        global = true,
        default_value = "info",
        env = "LOG_LEVEL",
        help = "the verbosity level to print logs at."
    )]
    pub log_level: LogLevel,

    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(self) -> operation_api_core::Result<()> {
        match self.command {
            Command::Generate(args) => {
                let gen_conf = operation_api_core::generate::GenerationConfig::new(
                    args.config.config_dir.as_deref(),
                )?;
                operation_api_core::generate::Generation::new(gen_conf)?
                    .generate_all(None)
                    .await
            },
            Command::Check(args) => {
                // we only need to initialize as we perform pre-checks at object creation
                let gen_conf = operation_api_core::generate::GenerationConfig::new(
                    args.config.config_dir.as_deref(),
                )?;
                let _ = operation_api_core::generate::Generation::new(gen_conf)?;
                Ok(())
            },
            Command::Init(args) => Ok(operation_api_manifests::init(args.name, args.dir)?),

            Command::Fmt(args) => {
                let targets =
                    operation_api_manifests::files::match_paths(&args.include, &args.exclude)?;

                Ok(
                    operation_api_parser::fmt::fmt(args.config.config_dir, targets, args.dry)
                        .await?,
                )
            },
        }
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Command {
    #[clap(alias = "gen", alias = "g")]
    /// generates models as defined in `op-gen.toml`
    Generate(GenArgs),

    #[clap(alias = "c")]
    /// checks models for soundness
    Check(CheckArgs),

    #[clap(alias = "i")]
    /// initializes a new schema project
    Init(InitArgs),

    #[clap(alias = "f")]
    /// formats schemas
    Fmt(FmtArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct WithConfig {
    #[clap(short = 'd', long = "config-dir")]
    config_dir: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
struct GenArgs {
    #[clap(flatten)]
    config: WithConfig,
}

#[derive(clap::Args, Debug, Clone)]
struct CheckArgs {
    #[clap(flatten)]
    config: WithConfig,
}

#[derive(clap::Args, Debug, Clone)]
struct InitArgs {
    #[clap(short = 'n', long, help = "the name of the package to create.")]
    name: String,
    #[clap(
        short = 'd',
        long,
        help = "the directory to create the new package in."
    )]
    dir: Option<PathBuf>,
}

#[derive(clap::Args, Debug, Clone)]
struct FmtArgs {
    #[clap(flatten)]
    config: WithConfig,

    #[clap(
        long,
        default_value_t = false,
        help = "if --dry, no edits will be written to files"
    )]
    dry: bool,

    #[clap(
        long,
        default_value_t = true,
        help = "if --safe=false, unsafe edits will be applied"
    )]
    safe: bool,

    #[clap(
        short,
        long,
        help = "a list of paths or globs to exclude from formatting."
    )]
    exclude: Vec<String>,

    #[clap(
        default_value = "./**/*.pld",
        help = "a list of paths or globs to include in formatting"
    )]
    include: Vec<String>,

    #[clap(short = 'W', long, help = "fail if warnings are encountered")]
    warn_is_fail: bool,
}
