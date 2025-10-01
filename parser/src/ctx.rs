use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock, RwLockReadGuard},
};

use miette::{Context, IntoDiagnostic};

use crate::{
    defs::{EnumDef, Ident, ImportDef, MessageDef, NamespaceDef, Payload, PayloadTypes, TypeDef},
    parser::PayloadParser,
    utils::insert_unique_ident_or_err,
};

pub trait GetCtx {
    fn get<'a>(
        &'a self,
        ident: &Ident,
    ) -> Option<PayloadTypes>;
    fn must_get<'a>(
        &'a self,
        ident: &Ident,
    ) -> crate::Result<PayloadTypes> {
        match self.get(ident) {
            Some(rf) => Ok(rf),
            None => Err(crate::Error::resolution(ident.clone())),
        }
    }
}

#[derive(Debug, bon::Builder, Clone)]
pub struct NamespaceCtx {
    pub source: PathBuf,

    pub namespace: NamespaceDef,

    pub types: HashMap<Ident, TypeDef>,

    pub messages: HashMap<Ident, MessageDef>,

    pub enums: HashMap<Ident, EnumDef>,

    pub imports: Vec<ImportDef>,
}

impl NamespaceCtx {
    pub fn is_part_of(
        &self,
        ns: &Ident,
    ) -> bool {
        Ident::path_equals(&self.namespace.ident, &ns.qualified_path())
    }

    fn merge<T>(
        ns: &NamespaceDef,
        values: &mut HashMap<Ident, T>,
        take_from: HashMap<Ident, T>,
    ) -> crate::Result<()> {
        for (ident, def) in take_from.into_iter() {
            match values.insert(ident.clone(), def) {
                Some(..) => {
                    return Err(crate::Error::conflict(ns.ident.clone(), ident));
                },
                None => {},
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
        Self::merge(&self.namespace, &mut self.messages, other.messages)?;
        Self::merge(&self.namespace, &mut self.enums, other.enums)?;

        self.imports.append(&mut other.imports);

        self.imports = self
            .imports
            .clone()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Ok(())
    }
}

impl GetCtx for NamespaceCtx {
    fn get<'a>(
        &'a self,
        ident: &Ident,
    ) -> Option<PayloadTypes> {
        if let Some(ty) = self.types.get(ident) {
            return Some(PayloadTypes::Type(ty.clone()));
        }

        if let Some(msg) = self.messages.get(ident) {
            return Some(PayloadTypes::Message(msg.clone()));
        }

        if let Some(enm) = self.enums.get(ident) {
            return Some(PayloadTypes::Enum(enm.clone()));
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

        let mut types = HashMap::new();
        let mut messages = HashMap::new();
        let mut enums = HashMap::new();

        let mut imports = Vec::new();

        let this = match value
            .defs
            .iter()
            .find(|ns| matches!(ns, PayloadTypes::Namespace(..)))
            .map(Clone::clone)
            .map(PayloadTypes::unwrap_namespace)
        {
            Some(def) => def,
            None => return Err(crate::Error::NsNotDeclared),
        };

        for v in value.defs {
            match v {
                PayloadTypes::Type(ty) => {
                    insert_unique_ident_or_err(
                        this.ident.clone(),
                        &mut types,
                        ty.ident.clone(),
                        ty,
                    )?
                },
                PayloadTypes::Message(msg) => {
                    insert_unique_ident_or_err(
                        this.ident.clone(),
                        &mut messages,
                        msg.ident.clone(),
                        msg,
                    )?
                },
                PayloadTypes::Enum(enm) => {
                    insert_unique_ident_or_err(
                        this.ident.clone(),
                        &mut enums,
                        enm.ident.clone(),
                        enm,
                    )?;
                },
                PayloadTypes::Import(import) => imports.push(import),
                PayloadTypes::Namespace(..) => {},
            }
        }

        Ok(namespace
            .types(types)
            .imports(imports)
            .messages(messages)
            .enums(enums)
            .namespace(this)
            .source(value.source)
            .build())
    }
}

#[derive(bon::Builder)]
pub struct Ctx {
    sources: RwLock<Vec<NamespaceCtx>>,
}

pub trait Parse {
    fn parse_file<P: AsRef<Path>>(
        &self,
        f: P,
    ) -> miette::Result<()>;

    fn parse_data<'p, P: Into<PathBuf>>(
        &self,
        source: P,
        s: &'p str,
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

    fn parse_data<'p, P: Into<PathBuf>>(
        &self,
        source: P,
        s: &'p str,
    ) -> miette::Result<()> {
        let source = source.into();
        if self
            .sources
            .read()
            .unwrap()
            .iter()
            .find(|it| it.source == source)
            .is_some()
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
        Self::builder()
            .sources(Default::default())
            .build()
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
    fn get<'a>(
        &'a self,
        ident: &Ident,
    ) -> Option<PayloadTypes> {
        let obj = ident.object();
        let find_ns = ident.namespace();

        let sources = self.sources.read().unwrap();
        for ns in sources.iter() {
            tracing::debug!("checking namespace: {:#?}", ns.namespace.ident);

            if !Ident::path_equals(&ns.namespace.ident, &find_ns) {
                continue;
            }
            if let Some(ty) = ns.types.get(&obj) {
                return Some(PayloadTypes::Type(ty.clone()));
            } else if let Some(msg) = ns.messages.get(&obj) {
                return Some(PayloadTypes::Message(msg.clone()));
            } else if let Some(enm) = ns.enums.get(&obj) {
                return Some(PayloadTypes::Enum(enm.clone()));
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

// pub fn parse_with_std<P: AsRef<std::path::Path>, I: IntoIterator<Item = P>>(
//     files: I,
// ) -> miette::Result<Ctx> {
//     let ctx = crate::payloads_std::get_std()?;
//     for f in files {
//         ctx.parse_file(f)?;
//     }
//     ctx.resolve_imports()?;
//     Ok(ctx)
// }

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

        assert!(matches!(msg, PayloadTypes::Message(..)));
    }
}
