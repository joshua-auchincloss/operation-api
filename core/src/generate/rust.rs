use std::{collections::BTreeMap, io::Write, path::PathBuf};

use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Lit, LitInt};

use crate::{
    Operation, Struct,
    generate::{GenOpts, Generate, LanguageTrait, RustConfig, context::WithNsContext},
};

pub struct RustGenerator;

impl LanguageTrait for RustGenerator {
    fn file_case() -> convert_case::Case<'static> {
        convert_case::Case::Snake
    }

    fn file_ext() -> &'static str {
        "rs"
    }
}

pub(crate) struct RustGenState {}

impl Generate<RustGenState, RustConfig> for RustGenerator {
    #[allow(unused)]
    fn on_create<'s>(
        state: &WithNsContext<'s, RustGenState, RustConfig, Self>,
        fname: &PathBuf,
        f: &mut Box<dyn Write>,
    ) -> std::io::Result<()> {
        Ok(())
    }

    #[allow(unused)]
    fn with_all_namespaces<'ns>(
        &self,
        ctx: &crate::context::Context,
        opts: &'ns GenOpts<RustConfig>,
        ctx_ns: BTreeMap<crate::Ident, WithNsContext<'ns, RustGenState, RustConfig, Self>>,
    ) -> super::Result<()> {
        for ns in ctx.namespaces.values() {
            let ctx = ctx_ns.get(&ns.name).unwrap();
            let mut tt = quote!();
            for it in ctx.ns.defs.keys() {
                let created = def_ident(it.clone());
                tt.extend(quote!(
                    #created,
                ))
            }
            let ns = ns.name.to_string();
            tt = quote!(operation_api_core::namespace! { #ns { #tt }});
            ctx.with_file_handle(ctx.ns_file(), |w| write!(w, "{tt}"))?;
        }
        Ok(())
    }

    #[allow(unused)]
    fn new_state<'ns>(
        &'ns self,
        opts: &GenOpts<RustConfig>,
    ) -> RustGenState {
        RustGenState {}
    }

    #[allow(unused)]
    fn gen_operation<'ns>(
        &self,
        state: &WithNsContext<'ns, RustGenState, RustConfig, Self>,
        def: &Operation,
    ) -> super::Result<()> {
        let ns_file = state.ns_file();
        let tt = quote::quote!();

        state.with_file_handle(ns_file, |w| {
            write!(w, "{tt}")?;
            Ok(())
        })?;
        Ok(())
    }

    #[allow(unused)]
    fn gen_struct<'ns>(
        &self,
        state: &WithNsContext<'ns, RustGenState, RustConfig, Self>,
        def: &Struct,
    ) -> super::Result<()> {
        let ns_file = state.ns_file();
        let mut tt = quote::quote!();
        let desc_comment = comment(&def.description);
        let op_comment = match &def.description {
            Some(desc) => quote::quote!(#[fields(describe(text = #desc))]),
            None => quote::quote!(),
        };
        let iden = ident(
            def.name
                .to_string()
                .to_case(convert_case::Case::Pascal),
        );
        let version = lit(format!("{}", def.version));

        let fields: TokenStream = def
            .fields
            .iter()
            .map(|(field_name, field)| {
                let f = ident(
                    field_name
                        .to_string()
                        .to_case(convert_case::Case::Snake),
                );
                let field = field.unwrap_field();
                let comment = comment(&field.description);
                let op_comment = match &field.description {
                    Some(desc) => {
                        quote::quote!(
                            #[fields(
                                describe(text = #desc)
                            )]
                        )
                    },
                    None => quote::quote!(),
                };
                let ty = field.ty.ty(&state.opts.opts);
                let name = field_name.clone().to_string();

                quote!(
                    #[serde(rename = #name)]
                    #op_comment
                    #comment
                    #f: #ty,
                )
            })
            .collect();

        tt.extend(quote::quote! {
            #[derive(serde::Serialize, serde::Deserialize, operation_api_sdk::Struct)]
            #[fields(version = #version)]
            #desc_comment
            #op_comment
            pub struct #iden {
                #fields
            }
        });

        tracing::info!("writing {} to '{}'", def.name, ns_file.display());

        state.with_file_handle(ns_file, |w| {
            write!(w, "{tt}")?;
            Ok(())
        })?;
        Ok(())
    }
}

fn def_ident(def: crate::Ident) -> Ident {
    ident(
        def.to_string()
            .to_case(convert_case::Case::Pascal),
    )
}

fn ident<D: AsRef<str>>(s: D) -> Ident {
    Ident::new(&s.as_ref(), proc_macro2::Span::call_site())
}

pub(crate) fn lit(value: String) -> TokenStream {
    let lit = Lit::Int(LitInt::new(&value, proc_macro2::Span::call_site()));
    quote! {#lit}
}

fn comment<T: ToTokens>(desc: &Option<T>) -> proc_macro2::TokenStream {
    match desc {
        Some(desc) => {
            quote!(
                #[doc = #desc]
            )
        },
        None => quote!(),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_ident() {
        super::ident("SomeStruct");
    }
}
