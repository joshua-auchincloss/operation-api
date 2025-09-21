use clap::Parser;
use human_panic::{Metadata, setup_panic};
use operation_api_cli::cli::Cli;

fn main() -> operation_api_core::Result<()> {
    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .homepage(env!("CARGO_PKG_HOMEPAGE"))
            .support(
                "Please open an issue on github. Attach the outputs of the above referenced report file."
            ).authors(env!("CARGO_PKG_AUTHORS"))
    );

    let cli = Cli::parse();

    let log_level: tracing::Level = cli.log_level.clone().into();

    let mut layer = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(log_level);

    #[cfg(test)]
    {
        layer = layer.with_file(true).with_line_number(true);
    }

    #[cfg(not(test))]
    {
        layer = layer
            .with_file(false)
            .with_line_number(false);
    }

    layer.init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");

    rt.block_on(cli.run())?;

    Ok(())
}
