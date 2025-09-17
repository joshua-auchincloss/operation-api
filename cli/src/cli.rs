#[derive(Default, clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Debug,
    Trace,
    #[default]
    Info,
    Error,
    Warn,
}

impl Into<tracing::Level> for LogLevel {
    fn into(self) -> tracing::Level {
        match self {
            Self::Debug => tracing::Level::DEBUG,
            Self::Trace => tracing::Level::TRACE,
            Self::Info => tracing::Level::INFO,
            Self::Error => tracing::Level::ERROR,
            Self::Warn => tracing::Level::WARN,
        }
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    #[clap(long, global = true, default_value = "info", env)]
    pub log_level: LogLevel,

    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(self) -> operation_api_core::Result<()> {
        match self.command {
            Command::Generate(args) => {
                let cfg = operation_api_core::generate::GenerationConfig::new(
                    args.config_dir.as_deref(),
                )?;
                cfg.generate_all().await
            },
        }
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Command {
    #[clap(alias = "gen")]
    Generate(GenArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct GenArgs {
    #[clap(short = 'd', long = "config-dir")]
    config_dir: Option<String>,
}
