use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Ident;

#[derive(darling::FromField)]
#[darling(attributes(fields))]
pub struct Field {
    ident: Option<Ident>,
    ty: syn::Type,

    describe: Option<DescOrPath>,
}

#[derive(darling::FromMeta, Clone)]
enum DescOrPath {
    Text(syn::LitStr),
    File(syn::LitStr),
}

fn ident<D: AsRef<str>>(s: D) -> Ident {
    Ident::from_string(&s.as_ref()).expect("new ident")
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

#[derive(darling::FromDeriveInput)]
#[darling(attributes(fields))]
pub struct Struct {
    ident: Ident,
    data: darling::ast::Data<(), Field>,

    version: usize,

    describe: Option<DescOrPath>,
}

#[derive(Debug)]
struct ResolvedDefs {
    desc: TokenStream,
    desc_value: TokenStream,
}
impl DescOrPath {
    fn resolve_defs(
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

pub fn derive_struct(tokens: TokenStream) -> TokenStream {
    let s = Struct::from_derive_input(&syn::parse(tokens.into()).unwrap()).unwrap();

    let desc = DescOrPath::resolve_defs(&s.ident, s.describe);

    let desc_value = desc.desc_value;
    let desc = desc.desc;

    let fields = s.data.take_struct().unwrap().fields;

    let fields_map = quote!(
        let mut m = std::collections::BTreeMap::<String, _>::new();
    );

    let parent_iden = s.ident.clone();

    let fields_def: TokenStream = fields
        .iter()
        .map(|field| {
            let iden = field
                .ident
                .clone()
                .expect("struct fields have idents");
            let ty = field.ty.clone();
            let iden_str = format!("{iden}");

            let desc = DescOrPath::resolve_defs(&iden, field.describe.clone());

            let desc_value = desc.desc_value;
            let desc = desc.desc;

            quote!(
                #desc

                m.insert(stringify!(#iden).into(), operation_api_core::Field{
                    ty: operation_api_core::ty!(#ty),
                    name: Some(#iden_str.into()),
                    namespace: Some(#parent_iden::NAMESPACE.into()),
                    description: #desc_value,
                    optional: false,
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

            operation_api_core::Definitions::StructV1(operation_api_core::Struct{
                name: #iden_lit.into(),
                namespace: #iden::NAMESPACE.into(),
                version: #version,
                description: #desc_value,
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
