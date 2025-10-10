fn main() {
    #[cfg(feature = "emit")]
    {
        use operation_api_cli::cli::Cli;

        let opts = clap_markdown::MarkdownOptions::new()
            .show_footer(false)
            .show_table_of_contents(true)
            .title("Cli Reference".into());
        let md = clap_markdown::help_markdown_custom::<Cli>(&opts);
        std::fs::write("docs/src/cli.md", md).unwrap();
    }
}
