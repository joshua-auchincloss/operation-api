use std::path::PathBuf;

use pest::iterators::Pairs;

use crate::{
    defs::{meta::MetaAttribute, span::Span},
    parser::Rule,
};

use crate::defs::*;

#[derive(Debug, Clone)]
pub enum PayloadTypes<V> {
    Import(Spanned<ImportDef>),
    Namespace(Spanned<NamespaceDef<V>>),
    Type(Spanned<TypeDef<V>>),
    Enum(Spanned<EnumDef<V>>),
    Struct(Spanned<StructDef<V>>),
}

pub type PayloadTypesSealed = PayloadTypes<usize>;
pub type PayloadTypesUnsealed = PayloadTypes<Option<usize>>;

impl<V> PayloadTypes<V> {
    pub fn span(&self) -> &Span {
        match self {
            Self::Namespace(s) => &s.span,
            Self::Type(s) => &s.span,
            Self::Struct(s) => &s.span,
            Self::Import(s) => &s.span,
            Self::Enum(s) => &s.span,
        }
    }

    pub fn version(&self) -> Option<&V> {
        match self {
            Self::Namespace(..) => None,
            Self::Import(..) => None,
            Self::Enum(enm) => Some(&enm.version),
            Self::Struct(def) => Some(&def.version),
            Self::Type(ty) => Some(&ty.version),
        }
    }

    pub fn unwrap_namespace(self) -> NamespaceDef<V> {
        match self {
            Self::Namespace(def) => def.value,
            _ => panic!("expected namespace"),
        }
    }

    pub fn unwrap_type(self) -> TypeDef<V> {
        match self {
            Self::Type(def) => def.value,
            _ => panic!("expected type"),
        }
    }

    pub fn unwrap_struct(self) -> StructDef<V> {
        match self {
            Self::Struct(def) => def.value,
            _ => panic!("expected struct"),
        }
    }

    pub fn unwrap_import(self) -> ImportDef {
        match self {
            Self::Import(def) => def.value,
            _ => panic!("expected import"),
        }
    }

    pub fn unwrap_enum(self) -> EnumDef<V> {
        match self {
            Self::Enum(def) => def.value,
            _ => panic!("expected enum"),
        }
    }
}

impl PayloadTypesUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> PayloadTypesSealed {
        match self {
            Self::Struct(s) => {
                PayloadTypesSealed::Struct(Spanned::new(
                    s.span.start,
                    s.span.end,
                    s.value.seal(file_version),
                ))
            },
            Self::Type(t) => {
                PayloadTypesSealed::Type(Spanned::new(
                    t.span.start,
                    t.span.end,
                    t.value.seal(file_version),
                ))
            },
            Self::Namespace(n) => {
                PayloadTypesSealed::Namespace(Spanned::new(
                    n.span.start,
                    n.span.end,
                    n.value.seal(file_version),
                ))
            },
            Self::Enum(e) => {
                PayloadTypesSealed::Enum(Spanned::new(
                    e.span.start,
                    e.span.end,
                    e.value.seal(file_version),
                ))
            },
            Self::Import(i) => PayloadTypesSealed::Import(i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub source: PathBuf,
    pub defs: Vec<PayloadTypesSealed>,
    pub outer_meta: Vec<Meta>,
    pub inner_meta: Vec<Meta>,
    pub version: Option<usize>,
}

impl Payload {
    pub fn build(
        source: PathBuf,
        pairs: Pairs<crate::parser::Rule>,
    ) -> crate::Result<Self> {
        let mut defs: Vec<PayloadTypesUnsealed> = vec![];

        let mut outer_meta: Vec<Meta> = Vec::new();
        let mut inner_meta: Vec<Meta> = Vec::new();
        let mut pending_outer_item_meta: Vec<Meta> = Vec::new();
        let mut pending_comment: Option<String> = None;

        for pair in pairs {
            let rule = pair.as_rule();
            tracing::trace!("rule: {rule:#?}");
            match rule {
                Rule::payloads => return Self::build(source, pair.into_inner()),
                Rule::type_def => {
                    tracing::trace!("found singular_typedef");
                    let mut ty = TypeDefUnsealed::from_pair_span(pair)?;
                    apply_pending_if_forward(&mut ty.value, &mut pending_comment);
                    apply_pending_meta(&mut ty.value.meta, &mut pending_outer_item_meta);

                    tracing::trace!("type {:#?}", ty.value);
                    defs.push(PayloadTypesUnsealed::Type(ty));
                },
                Rule::struct_def => {
                    tracing::trace!("found struct");
                    let mut st = StructUnsealed::from_pair_span(pair)?;
                    apply_pending_if_forward(&mut st.value, &mut pending_comment);
                    apply_pending_meta(&mut st.value.meta, &mut pending_outer_item_meta);

                    tracing::trace!("struct {:#?}", st.value);
                    defs.push(PayloadTypesUnsealed::Struct(st));
                },
                Rule::import_def => {
                    tracing::trace!("found import");
                    let mut import = ImportDef::from_pair_span(pair)?;
                    apply_pending_if_forward(&mut import.value, &mut pending_comment);
                    apply_pending_meta(&mut import.value.meta, &mut pending_outer_item_meta);

                    tracing::trace!("import {:#?}", import.value);
                    defs.push(PayloadTypesUnsealed::Import(import));
                },
                Rule::namespace_def => {
                    tracing::trace!("found namespace");
                    let mut ns = NamespaceUnsealed::from_pair_span(pair)?;
                    apply_pending_if_forward(&mut ns.value, &mut pending_comment);
                    apply_pending_meta(&mut ns.value.meta, &mut pending_outer_item_meta);

                    tracing::trace!("namespace {:#?}", ns.value);
                    defs.push(PayloadTypesUnsealed::Namespace(ns));
                },
                Rule::enum_def => {
                    tracing::trace!("found enum");
                    let mut enum_def = EnumUnsealed::from_pair_span(pair)?;
                    apply_pending_if_forward(&mut enum_def.value, &mut pending_comment);
                    apply_pending_meta(&mut enum_def.value.meta, &mut pending_outer_item_meta);

                    tracing::trace!("enum {:#?}", enum_def.value);
                    defs.push(PayloadTypesUnsealed::Enum(enum_def));
                },
                Rule::multiline_comment | Rule::comment => {
                    tracing::trace!("found multiline comment");
                    pending_comment = Some(take_comment(Pairs::single(pair.clone())));
                },
                Rule::inner_meta => {
                    inner_meta.push(Meta::parse(MetaAttribute::from_pair_span(pair)?)?);
                },
                Rule::outer_meta => {
                    let meta = Meta::parse(MetaAttribute::from_pair_span(pair)?)?;
                    if defs.is_empty() {
                        outer_meta.push(meta);
                    } else {
                        pending_outer_item_meta.push(meta);
                    }
                },
                Rule::singleline_comment => {
                    tracing::trace!("found single line comment");
                    let comment = take_comment(Pairs::single(pair.clone()));
                    tracing::info!("comment: '{:#?}' '{comment:#?}'", pair.as_str());

                    let last = if defs.is_empty() {
                        None
                    } else {
                        let sz = defs.len() - 1;
                        defs.get_mut(sz)
                    };

                    match last {
                        Some(last) => {
                            match last {
                                PayloadTypesUnsealed::Import(import) => {
                                    tracing::trace!(
                                        "commenting on import: '{comment}' -> {import:#?}"
                                    );
                                    import.value.comment(comment);
                                },
                                PayloadTypesUnsealed::Struct(st) => {
                                    tracing::trace!("commenting on struct: '{comment}' -> {st:#?}");
                                    st.value.comment(comment);
                                },
                                PayloadTypesUnsealed::Namespace(ns) => {
                                    tracing::trace!("commenting on ns: '{comment}' -> {ns:#?}");
                                    ns.value.comment(comment);
                                },
                                PayloadTypesUnsealed::Type(ty) => {
                                    tracing::trace!("commenting on ns: '{comment}' -> {ty:#?}");
                                    ty.value.comment(comment);
                                },
                                PayloadTypesUnsealed::Enum(enm) => {
                                    tracing::trace!("commenting on enum: '{comment}' -> {enm:#?}");
                                    enm.value.comment(comment);
                                },
                            }
                        },
                        None => {
                            pending_comment = Some(comment);
                        },
                    }
                },

                Rule::EOI => {
                    break;
                },
                rule => {
                    tracing::error!("unhandled rule: {rule:#?}");
                    let sp = pair.as_span();
                    return Err(crate::Error::defs::<Self, _>([
                        Rule::import_def,
                        Rule::struct_def,
                        Rule::type_def,
                        Rule::payloads,
                    ])
                    .with_span(sp.start(), sp.end()));
                },
            }
        }
        tracing::trace!("done");

        let version = resolve_version(&defs, &outer_meta, &inner_meta)?;
        let sealed_defs: Vec<PayloadTypesSealed> = if let Some(v) = version {
            defs.into_iter().map(|d| d.seal(v)).collect()
        } else {
            defs.into_iter().map(|d| d.seal(0)).collect()
        };

        Ok(Payload {
            source,
            defs: sealed_defs,
            version,
            outer_meta,
            inner_meta,
        })
    }

    pub fn version(&self) -> Option<usize> {
        self.version
    }

    pub fn outer_file_meta(&self) -> &[Meta] {
        &self.outer_meta
    }

    pub fn inner_file_meta(&self) -> &[Meta] {
        &self.inner_meta
    }
}

mod test {}

fn resolve_version(
    defs: &[PayloadTypesUnsealed],
    outer_meta: &[Meta],
    inner_meta: &[Meta],
) -> crate::Result<Option<usize>> {
    use crate::utils::{detect_version_conflict, extract_versions};

    let outer_versions = extract_versions(outer_meta);
    let outer_val = detect_version_conflict(&outer_versions)?;

    let inner_val = if outer_val.is_none() {
        let inner_versions = extract_versions(inner_meta);
        detect_version_conflict(&inner_versions)?
    } else {
        None
    };

    for def in defs {
        let metas: &Vec<Meta> = match &def {
            PayloadTypesUnsealed::Struct(s) => &s.value.meta,
            PayloadTypesUnsealed::Type(t) => &t.value.meta,
            PayloadTypesUnsealed::Import(i) => &i.value.meta,
            PayloadTypesUnsealed::Namespace(n) => &n.value.meta,
            PayloadTypesUnsealed::Enum(e) => &e.value.meta,
        };
        let versions = extract_versions(metas);
        // ignore the returned canonical value here; we only care if an error is raised.
        let _ = detect_version_conflict(&versions)?;
    }

    Ok(outer_val.or(inner_val))
}
