use crate::shared::*;
use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, spanned::Spanned};

include!("macros.rs");

#[derive(darling::FromField)]
#[darling(attributes(fields))]
pub struct Field {
    ident: Option<Ident>,
    ty: syn::Type,

    #[darling(default)]
    enm: bool,

    #[darling(default)]
    one_of: bool,

    describe: Option<DescOrPath>,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(fields))]
pub struct Struct {
    ident: Ident,
    data: darling::ast::Data<(), Field>,

    version: usize,

    describe: Option<DescOrPath>,
}

pub fn derive_struct(tokens: TokenStream) -> TokenStream {
    let input = call_span!(syn::parse(tokens.clone().into()));

    let s = match Struct::from_derive_input(&input) {
        Ok(s) => s,
        Err(e) => return e.write_errors(),
    };

    let desc = DescOrPath::resolve_defs(&s.ident, s.describe);

    let desc_value = desc.desc_value;
    let desc = desc.desc;

    let fields = call_span!(
        @opt s.data.take_struct();
        syn::Error::new(s.ident.span(), "#[derive(Struct)] can only be used on structs")
    )
    .fields;

    let fields_map = quote!(
        let mut m = std::collections::BTreeMap::<_, _>::new();
    );

    let parent_iden = s.ident.clone();

    if let Some(bad) = fields.iter().find(|f| f.ident.is_none()) {
        let err: syn::Result<()> = Err(syn::Error::new(
            bad.ty.span(),
            "struct fields must be named (no tuple or unit fields)",
        ));
        call_span!(err);
    }

    let mut fields_def = quote!();
    for field in &fields {
        let Some(iden) = field.ident.clone() else {
            continue;
        };

        if field.enm && field.one_of {
            return syn::Error::new(
                field.ty.span(), "cannot have both enum and one_of types. if you are using a literal, use enum. if you are using type discriminants, use one_of."
            ).into_compile_error();
        }

        let ty = field.ty.clone();
        let iden_str = format!("{iden}");

        let desc = DescOrPath::resolve_defs(&iden, field.describe.clone());

        let desc_value = desc.desc_value;
        let desc = desc.desc;

        let ty_clause = if field.enm {
            quote!(@enm)
        } else if field.one_of {
            quote!(@one_of)
        } else {
            quote!()
        };

        fields_def.extend(quote!(
            #desc

            m.insert(stringify!(#iden).into(), operation_api_core::Field{
                meta: operation_api_core::Meta {
                    name: Some(#iden_str.into()),
                    namespace: Some(#parent_iden::NAMESPACE.into()),
                    description: #desc_value,
                    version: None,
                },
                ty: operation_api_core::ty!(#ty_clause #ty),
                optional: false,
            }.into());
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
            operation_api_core::Definitions::StructV1(operation_api_core::Struct{
                meta: operation_api_core::Meta {
                    name: #iden_lit.into(),
                    namespace: #iden::NAMESPACE.into(),
                    version: VERSION.into(),
                    description: #desc_value,
                },
                fields: operation_api_core::FieldsList::new(m),
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
