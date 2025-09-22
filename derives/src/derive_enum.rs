use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, Lit, LitStr};

use crate::shared::*;

include!("macros.rs");

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
    let input = call_span!(syn::parse(tokens.clone().into()));
    let s = match Enum::from_derive_input(&input) {
        Ok(s) => s,
        Err(e) => return e.write_errors(),
    };

    let desc = DescOrPath::resolve_defs(&s.ident, s.describe);

    let desc_value = desc.desc_value;
    let desc = desc.desc;

    let fields = call_span!(
        @opt s.data.take_enum();
        syn::Error::new(s.ident.span(), "#[derive(Enum)] can only be used on enums")
    );

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
                    quote!(operation_api_sdk::StrOrInt::String(#value.into()))
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
                                                operation_api_sdk::StrOrInt::Int(value)
                                            })
                                        },
                                        lit => panic!("{lit:#?} type is unsupported"),
                                    }
                                },
                                _ => panic!("only lit exprs are permitted"),
                            }
                        },
                        None => quote!(operation_api_sdk::StrOrInt::Int(#i)),
                    }
                },
            };

            quote!(
                #desc

                m.insert(#iden_str.into(), operation_api_sdk::VariantKind{
                    meta: operation_api_sdk::Meta {
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
        static #iden_def: std::sync::LazyLock<operation_api_sdk::Definitions> = std::sync::LazyLock::new(|| {
            use operation_api_sdk::OfNamespace;

            #fields_map
            #fields_def

            const VERSION: operation_api_sdk::Version = operation_api_sdk::Version::new(#version);
            operation_api_sdk::Definitions::EnumV1(operation_api_sdk::Enum{
                meta: operation_api_sdk::Meta {
                    name: #iden_lit.into(),
                    namespace: #iden::NAMESPACE.into(),
                    version: VERSION.into(),
                    description: #desc_value,
                },
                variants: operation_api_sdk::Named::new(m),
            })
        });

        impl operation_api_sdk::Typed for #iden {
            fn ty() -> operation_api_sdk::Type {
                operation_api_sdk::Type::CompoundType(
                    operation_api_sdk::CompoundType::Enum{
                        to: #iden_lit.into()
                    }
                )
            }
        }

        impl operation_api_sdk::Defined for #iden {
            fn definition() -> &'static operation_api_sdk::Definitions {
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
