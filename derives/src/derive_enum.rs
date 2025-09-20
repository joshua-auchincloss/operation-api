use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, Lit, LitStr};

use crate::shared::*;

#[derive(darling::FromVariant)]
#[darling(attributes(fields))]
pub struct Field {
    ident: Ident,
    discriminant: Option<syn::Expr>,

    #[darling(default)]
    describe: Option<DescOrPath>,

    #[darling(default)]
    str_value: Option<LitStr>,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(fields), supports(enum_any))]
pub struct Enum {
    ident: Ident,
    data: darling::ast::Data<Field, ()>,

    version: usize,

    describe: Option<DescOrPath>,
}

pub fn derive_enum(tokens: TokenStream) -> TokenStream {
    let s = Enum::from_derive_input(&syn::parse(tokens.into()).expect("syn parse"))
        .expect("darling parse");

    let desc = DescOrPath::resolve_defs(&s.ident, s.describe);

    let desc_value = desc.desc_value;
    let desc = desc.desc;

    let fields = s.data.take_enum().expect("enum");

    let fields_map = quote!(
        let mut m = std::collections::BTreeMap::<_, _>::new();
    );

    let parent_iden = s.ident.clone();

    let fields_def: TokenStream = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let iden = field.ident.clone();
            let iden_str = format!("{iden}");

            let desc = DescOrPath::resolve_defs(&iden, field.describe.clone());

            let desc_value = desc.desc_value;
            let desc = desc.desc;
            let value = match &field.str_value {
                Some(value) => {
                    quote!(operation_api_core::StrOrInt::String(#value.into()))
                },
                None => {
                    match field.discriminant.clone() {
                        Some(expr) => {
                            match expr {
                                Expr::Lit(lit) => {
                                    match lit.lit {
                                        Lit::Int(int) => {
                                            quote!({
                                                let value: usize = #int;
                                                operation_api_core::StrOrInt::Int(value)
                                            })
                                        },
                                        lit => panic!("{lit:#?} type is unsupported"),
                                    }
                                },
                                _ => panic!("only lit exprs are permitted"),
                            }
                        },
                        None => quote!(operation_api_core::StrOrInt::Int(#i)),
                    }
                },
            };

            quote!(
                #desc

                m.insert(#iden_str.into(), operation_api_core::VariantKind{
                    meta: operation_api_core::Meta {
                        name: #iden_str.into(),
                        namespace: Some(#parent_iden::NAMESPACE.into()),
                        description: #desc_value,
                        version: None,
                    },
                    value: #value
                }.into());
            )
        })
        .collect();

    let iden = s.ident.clone();
    let iden_lit = iden.to_string();
    let version = s.version;
    let iden_def = ident(format!("{iden}_DEF").to_case(Case::UpperSnake));

    let def = quote!(
        static #iden_def: std::sync::LazyLock<operation_api_core::Definitions> = std::sync::LazyLock::new(|| {
            use operation_api_core::namespace::OfNamespace;

            #fields_map
            #fields_def

            const VERSION: operation_api_core::Version = operation_api_core::Version::new(#version);
            operation_api_core::Definitions::EnumV1(operation_api_core::Enum{
                meta: operation_api_core::Meta {
                    name: #iden_lit.into(),
                    namespace: #iden::NAMESPACE.into(),
                    version: VERSION.into(),
                    description: #desc_value,
                },
                variants: operation_api_core::Named::new(m),
            })
        });


        impl operation_api_core::Defined for #iden {
            fn definition() -> &'static operation_api_core::Definitions {
                use std::ops::Deref;
                #iden_def.deref()
            }
        }
    );
    quote! {
        #desc

        #def
    }
}
