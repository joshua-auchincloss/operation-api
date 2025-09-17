use std::path::PathBuf;

use operation_api_core::Definitions;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Token, parse::Parse};

struct Attributes {
    src: PathBuf,
}

impl Parse for Attributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let path: LitStr = input.parse()?;

        Ok(Self {
            src: path.value().into(),
        })
    }
}

pub fn generate_module(
    attr: TokenStream,
    _tokens: TokenStream,
) -> TokenStream {
    let atts: Attributes = syn::parse2(attr).unwrap();

    let src = std::fs::read_to_string(
        std::env::current_dir()
            .expect("cwd")
            .join(&atts.src),
    )
    .unwrap();

    let _def: Definitions = match atts
        .src
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
    {
        "yaml" | "yml" => serde_yaml::from_str(&src).unwrap(),
        "json" => serde_json::from_str(&src).unwrap(),
        "toml" => toml::from_str(&src).unwrap(),
        ext => unimplemented!("{ext} is not implemented"),
    };

    quote! {}
}
