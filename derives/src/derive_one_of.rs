use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, ast::Fields};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::shared::*;

include!("macros.rs");

#[derive(darling::FromVariant)]
#[darling(attributes(fields))]
pub struct OneOfField {
    ident: Ident,
    fields: Fields<OneOfInnerField>,
}

#[derive(FromField, Clone)]
pub struct OneOfInnerField {
    pub ty: Type,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(fields), supports(enum_any))]
pub struct OneOfDesc {
    ident: Ident,
    data: darling::ast::Data<OneOfField, ()>,

    version: usize,

    #[darling(default)]
    describe: Option<DescOrPath>,
}

pub fn derive_one_of(tokens: TokenStream) -> TokenStream {
    let input = call_span!(syn::parse(tokens.clone().into()));
    let s = match OneOfDesc::from_derive_input(&input) {
        Ok(s) => s,
        Err(e) => return e.write_errors(),
    };

    let desc = DescOrPath::resolve_defs(&s.ident, s.describe);
    let desc_value = desc.desc_value;
    let desc = desc.desc;

    let fields = call_span!(
        @opt s.data.take_enum();
        syn::Error::new(s.ident.span(), "#[derive(OneOf)] can only be used on enums")
    );

    let fields_map = quote!(
        let mut m = std::collections::BTreeMap::<_, operation_api_core::OneOfVariant>::new();
    );

    let mut fields_def = quote!();
    let mut saw_nullish = false;
    for field in fields {
        let iden = field.ident.clone();
        let iden_str = format!("{iden}");
        let ty_tokens = match field
            .fields
            .fields
            .iter()
            .map(|it| &it.ty)
            .next()
        {
            Some(Type::Path(pat)) => quote::quote!(#pat),
            Some(..) => {
                return syn::Error::new(
                    field.ident.span(),
                    "a type path must be given for one_of variants",
                )
                .into_compile_error();
            },
            None => {
                if saw_nullish {
                    return syn::Error::new(
                        field.ident.span(),
                        "only one field variant may be assigned a zero value.",
                    )
                    .into_compile_error();
                }
                saw_nullish = true;
                quote::quote! {Never}
            },
        };

        fields_def.extend(quote!(
            m.insert(#iden_str.into(), operation_api_core::OneOfVariant{
                name: #iden_str.into(),
                ty: operation_api_core::ty!(#ty_tokens),
            });
        ));
    }

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
            operation_api_core::Definitions::OneOfV1(operation_api_core::OneOf{
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
