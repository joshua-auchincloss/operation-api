use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{RwLock, RwLockReadGuard},
};

use miette::{Context, IntoDiagnostic};

use crate::{
    defs::{
        EnumSealed, Ident, ImportDef, NamespaceSealed, Payload, PayloadTypesSealed, StructSealed,
        TypeDefSealed,
    },
    parser::PayloadParser,
    utils::insert_unique_ident_or_err_spanned,
};

pub trait GetCtx {
    fn get(
        &self,
        ident: &Ident,
    ) -> Option<PayloadTypesSealed>;
    fn get_spanned(
        &self,
        ident: &crate::defs::Spanned<Ident>,
    ) -> Option<PayloadTypesSealed> {
        self.get(&ident.value)
    }
    fn must_get(
        &self,
        ident: &crate::defs::Spanned<Ident>,
    ) -> crate::Result<PayloadTypesSealed> {
        match self.get_spanned(ident) {
            Some(rf) => Ok(rf),
            None => {
                Err(crate::Error::resolution_spanned(
                    ident.value.clone(),
                    ident.span.start,
                    ident.span.end,
                ))
            },
        }
    }
}

#[derive(Debug, bon::Builder, Clone)]
pub struct NamespaceCtx {
    pub source: PathBuf,

    pub namespace: crate::defs::Spanned<NamespaceSealed>,

    pub types: HashMap<Ident, crate::defs::Spanned<TypeDefSealed>>,

    pub structs: HashMap<Ident, crate::defs::Spanned<StructSealed>>,

    pub enums: HashMap<Ident, crate::defs::Spanned<EnumSealed>>,

    pub imports: Vec<crate::defs::Spanned<ImportDef>>,
}

impl NamespaceCtx {
    pub fn is_part_of(
        &self,
        ns: &Ident,
    ) -> bool {
        Ident::path_equals(&self.namespace.ident, &ns.qualified_path())
    }

    fn merge<T>(
        ns: &NamespaceSealed,
        values: &mut HashMap<Ident, T>,
        take_from: HashMap<Ident, T>,
    ) -> crate::Result<()> {
        for (ident, def) in take_from.into_iter() {
            if values.insert(ident.clone(), def).is_some() {
                return Err(crate::Error::conflict(ns.ident.clone(), ident));
            }
        }
        Ok(())
    }

    pub fn join(
        &mut self,
        other: Self,
    ) -> crate::Result<()> {
        let mut other = other;

        Self::merge(&self.namespace, &mut self.types, other.types)?;
        Self::merge(&self.namespace, &mut self.structs, other.structs)?;
        Self::merge(&self.namespace, &mut self.enums, other.enums)?;

        self.imports.append(&mut other.imports);

        // deduplicate imports by path (keep first occurrence span)
        let mut seen = HashSet::new();
        self.imports.retain(|imp| {
            let key = &imp.value.path;
            if seen.contains(key) {
                false
            } else {
                seen.insert(key.clone());
                true
            }
        });

        Ok(())
    }
}

impl GetCtx for NamespaceCtx {
    fn get(
        &self,
        ident: &Ident,
    ) -> Option<PayloadTypesSealed> {
        if let Some(ty) = self.types.get(ident) {
            return Some(PayloadTypesSealed::Type(ty.clone()));
        }

        if let Some(msg) = self.structs.get(ident) {
            return Some(PayloadTypesSealed::Struct(msg.clone()));
        }

        if let Some(enm) = self.enums.get(ident) {
            return Some(PayloadTypesSealed::Enum(enm.clone()));
        }
        None
    }
}

impl NamespaceCtx {
    fn resolve_imports<P: Parse>(
        &self,
        ctx: &P,
    ) -> miette::Result<()> {
        for import in &self.imports {
            ctx.parse_file(
                self.source
                    .parent()
                    .expect("non root dir")
                    .join(&import.path),
            )?;
        }
        Ok(())
    }
}
impl TryFrom<Payload> for NamespaceCtx {
    type Error = crate::Error;
    fn try_from(value: Payload) -> crate::Result<Self> {
        let namespace = Self::builder();

        let mut types: HashMap<Ident, crate::defs::Spanned<TypeDefSealed>> = HashMap::new();
        let mut structs: HashMap<Ident, crate::defs::Spanned<StructSealed>> = HashMap::new();
        let mut enums: HashMap<Ident, crate::defs::Spanned<EnumSealed>> = HashMap::new();

        let mut imports = Vec::new();

        let ns_spanned = match value
            .defs
            .iter()
            .find(|ns| matches!(ns, PayloadTypesSealed::Namespace(..)))
            .cloned()
        {
            Some(PayloadTypesSealed::Namespace(ns)) => ns,
            _ => return Err(crate::Error::NsNotDeclared),
        };
        let this_ident_path = ns_spanned.value.ident.clone();

        for v in value.defs {
            match v {
                PayloadTypesSealed::Type(ty) => {
                    let (start, end, ident) = (ty.span.start, ty.span.end, ty.value.ident.clone());
                    let clone_for_insert = ty.clone();
                    insert_unique_ident_or_err_spanned(
                        this_ident_path.clone(),
                        &mut types,
                        ident,
                        clone_for_insert,
                        start,
                        end,
                    )?
                },
                PayloadTypesSealed::Struct(msg) => {
                    let (start, end, ident) =
                        (msg.span.start, msg.span.end, msg.value.ident.clone());
                    let clone_for_insert = msg.clone();
                    insert_unique_ident_or_err_spanned(
                        this_ident_path.clone(),
                        &mut structs,
                        ident,
                        clone_for_insert,
                        start,
                        end,
                    )?
                },
                PayloadTypesSealed::Enum(enm) => {
                    let clone_for_insert = enm.clone();
                    insert_unique_ident_or_err_spanned(
                        this_ident_path.clone(),
                        &mut enums,
                        enm.value.ident.clone(),
                        clone_for_insert,
                        enm.span.start,
                        enm.span.end,
                    )?;
                },
                PayloadTypesSealed::Import(import) => imports.push(import),
                PayloadTypesSealed::Namespace(..) => {},
            }
        }

        Ok(namespace
            .types(types)
            .imports(imports)
            .structs(structs)
            .enums(enums)
            .namespace(ns_spanned)
            .source(value.source)
            .build())
    }
}

#[derive(bon::Builder, Default)]
pub struct Ctx {
    sources: RwLock<Vec<NamespaceCtx>>,
}

pub trait Parse {
    fn parse_file<P: AsRef<Path>>(
        &self,
        f: P,
    ) -> miette::Result<()>;

    fn parse_data<P: Into<PathBuf>>(
        &self,
        source: P,
        s: &str,
    ) -> miette::Result<()>;
}

impl Parse for Ctx {
    fn parse_file<P: AsRef<Path>>(
        &self,
        f: P,
    ) -> miette::Result<()> {
        let data = std::fs::read_to_string(f.as_ref())
            .into_diagnostic()
            .wrap_err("failed to read source file")?;
        self.parse_data(f.as_ref(), &data)?;
        Ok(())
    }

    fn parse_data<P: Into<PathBuf>>(
        &self,
        source: P,
        s: &str,
    ) -> miette::Result<()> {
        let source = source.into();
        if self
            .sources
            .read()
            .unwrap()
            .iter()
            .any(|it| it.source == source)
        {
            return Ok(());
        }

        self.sources.write().unwrap().push(
            PayloadParser::parse_data(source, s)?
                .try_into()
                .into_diagnostic()
                .wrap_err("failed to parse payload")?,
        );

        Ok(())
    }
}

impl Ctx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn namespaces(&self) -> RwLockReadGuard<'_, Vec<NamespaceCtx>> {
        self.sources.read().unwrap()
    }

    pub fn resolve_imports(&self) -> miette::Result<()> {
        let sources = self.sources.read().unwrap().clone();

        sources
            .iter()
            .try_for_each(|ns| ns.resolve_imports(self))?;
        Ok(())
    }
}

impl GetCtx for Ctx {
    fn get(
        &self,
        ident: &Ident,
    ) -> Option<PayloadTypesSealed> {
        let obj = ident.object();
        let find_ns = ident.namespace();

        let sources = self.sources.read().unwrap();
        for ns in sources.iter() {
            tracing::debug!("checking namespace: {:#?}", ns.namespace.ident);

            if !Ident::path_equals(&ns.namespace.ident, &find_ns) {
                continue;
            }
            if let Some(ty) = ns.types.get(&obj) {
                return Some(PayloadTypesSealed::Type(ty.clone()));
            } else if let Some(msg) = ns.structs.get(&obj) {
                return Some(PayloadTypesSealed::Struct(msg.clone()));
            } else if let Some(enm) = ns.enums.get(&obj) {
                return Some(PayloadTypesSealed::Enum(enm.clone()));
            }
        }
        None
    }
}

pub fn parse_files<I: IntoIterator<Item = P>, P: AsRef<Path>>(files: I) -> miette::Result<Ctx> {
    let ctx = Ctx::new();
    for f in files {
        ctx.parse_file(f)?;
    }
    ctx.resolve_imports()?;
    Ok(ctx)
}

#[cfg(test)]
mod test {
    use crate::tst::logging;

    use super::*;

    #[test]
    fn test_parse() {
        logging();

        let ctx = parse_files(vec!["samples/test_message.pld"]).unwrap();

        let msg = ctx
            .get(&"test::test_message".into())
            .expect("should find TestMessage");

        assert!(matches!(msg, PayloadTypesSealed::Struct(..)));
    }
}
