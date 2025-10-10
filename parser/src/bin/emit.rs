pub fn main() {
    #[cfg(feature = "emit")]
    {
        operation_api_parser::tokens::toks::emit_syntax("syntax.json");
    }
    #[cfg(not(feature = "emit"))]
    panic!("cannot emit without feature enabled")
}
