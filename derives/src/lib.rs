mod derive_enum;
mod derive_one_of;
mod derive_struct;
mod generate_module;
mod generate_ops;
pub(crate) mod shared;

use proc_macro::TokenStream;

#[proc_macro_derive(Enum, attributes(fields))]
pub fn derive_enum(tokens: TokenStream) -> TokenStream {
    derive_enum::derive_enum(tokens.into()).into()
}

#[proc_macro_derive(Struct, attributes(fields))]
pub fn derive_struct(tokens: TokenStream) -> TokenStream {
    derive_struct::derive_struct(tokens.into()).into()
}

#[proc_macro_derive(OneOf, attributes(fields))]
pub fn derive_one_of(tokens: TokenStream) -> TokenStream {
    derive_one_of::derive_one_of(tokens.into()).into()
}

#[proc_macro_attribute]
pub fn module(
    attr: TokenStream,
    tokens: TokenStream,
) -> TokenStream {
    generate_module::generate_module(attr.into(), tokens.into()).into()
}

#[proc_macro_attribute]
pub fn operations(
    _attr: TokenStream,
    _tokens: TokenStream,
) -> TokenStream {
    // generate_ops::operations(attr.into(), tokens.into()).into()
    todo!()
}
