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
pub struct Cli {
    #[clap(long, global = true, default_value = "info", env = "LOG_LEVEL")]
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
        }
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Command {
    #[clap(alias = "gen", alias = "g")]
    Generate(GenArgs),

    #[clap(alias = "c")]
    Check(CheckArgs),
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
