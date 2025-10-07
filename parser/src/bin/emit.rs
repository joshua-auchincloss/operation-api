pub fn main() {
    #[cfg(feature = "emit")]
    {
        operation_api_parser::tokens::toks::emit_syntax("syntax.json");
        operation_api_parser::fmt::RuleCollector::new().emit_rules("rules.json");
    }
    #[cfg(not(feature = "emit"))]
    panic!("cannot emit without feature enabled")
}
