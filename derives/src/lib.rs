mod derive_struct;
mod generate_module;

use proc_macro::TokenStream;

#[proc_macro_derive(Struct, attributes(fields))]
pub fn derive_struct(tokens: TokenStream) -> TokenStream {
    derive_struct::derive_struct(tokens.into()).into()
}

#[proc_macro_attribute]
pub fn module(
    attr: TokenStream,
    tokens: TokenStream,
) -> TokenStream {
    generate_module::generate_module(attr.into(), tokens.into()).into()
}
