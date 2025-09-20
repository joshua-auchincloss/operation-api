use std::path::PathBuf;

use operation_api_core::generate::{Generation, GenerationConfig, files::MemCollector};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr, Visibility, braced, parse::Parse};

use crate::shared::ident;

struct Attributes {
    src: PathBuf,
}

impl Parse for Attributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: LitStr = input
            .parse()
            .unwrap_or_else(|_| LitStr::new("./", Span::call_site()));
        Ok(Self {
            src: path.value().into(),
        })
    }
}

struct Module {
    vis: Visibility,
    name: Ident,
    contents: TokenStream,
}

impl Parse for Module {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = match input.parse() {
            Ok(vis) => vis,
            Err(_) => Visibility::Inherited,
        };
        let _: syn::token::Mod = input.parse()?;

        let name = input.parse()?;
        let tokens;
        braced!(tokens in input);
        let contents = tokens.parse()?;
        Ok(Self {
            vis,
            name,
            contents,
        })
    }
}

pub fn generate_module(
    attr: TokenStream,
    tokens: TokenStream,
) -> TokenStream {
    let atts: Attributes = syn::parse2(attr).unwrap();
    let module: Module = syn::parse2(tokens).unwrap();

    let src = std::env::current_dir()
        .expect("cwd")
        .join(&atts.src);

    let mut conf =
        GenerationConfig::new(Some(src.display().to_string().as_str())).expect("generation config");

    conf.set_mem(true);

    let generation = Generation::new(conf).expect("context");

    let collector = MemCollector::new();

    generation
        .generate_all_sync(Some(collector.mem_flush()))
        .expect("generate sync");

    let generated = collector.files();

    let mut outputs = quote!();

    for (entry, data) in generated.iter() {
        let data = String::from_utf8_lossy(&data);
        // we can use the file name as the mod name as we already converted to form when generating
        let mod_name = ident(
            entry
                .file_name()
                .expect("file name")
                .to_str()
                .unwrap()
                .replace(".rs", ""),
        );
        let mod_data: TokenStream = syn::parse_str(&data).expect("parse str");
        outputs.extend(quote! {
            pub mod #mod_name {
                #mod_data
            }
        })
    }

    let vis = module.vis;
    let mod_name = module.name;
    let contents = module.contents;
    quote! {
        #vis mod #mod_name {
            #outputs
            #contents
        }
    }
}
