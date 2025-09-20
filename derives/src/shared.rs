use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Ident;

pub fn ident<D: AsRef<str>>(s: D) -> Ident {
    Ident::from_string(&s.as_ref()).expect("new ident")
}

#[derive(darling::FromMeta, Clone)]
pub enum DescOrPath {
    Text(syn::LitStr),
    File(syn::LitStr),
}

impl DescOrPath {
    pub fn resolve_defs(
        parent: &Ident,
        this: Option<Self>,
    ) -> ResolvedDefs {
        let mut desc_iden: Option<Ident> = None;

        let desc = if let Some(desc) = this {
            let iden = ident(&format!("{}_DESCRIPTION", parent).to_case(Case::UpperSnake));
            desc_iden = Some(iden.clone());
            quote! {
                const #iden: &'static str = #desc;
            }
        } else {
            quote!()
        };

        let desc_value = match desc_iden {
            Some(iden) => {
                quote!(Some(#iden.into()))
            },
            None => {
                quote!(None)
            },
        };

        ResolvedDefs { desc, desc_value }
    }
}

impl ToTokens for DescOrPath {
    fn to_tokens(
        &self,
        tokens: &mut TokenStream,
    ) {
        tokens.extend(match self {
            Self::File(file) => {
                let value = file.value();
                quote!(include_str!(#value))
            },
            Self::Text(lit) => {
                let value = lit.value();
                quote!(#value)
            },
        });
    }
}

#[derive(Debug)]
pub struct ResolvedDefs {
    pub desc: TokenStream,
    pub desc_value: TokenStream,
}
