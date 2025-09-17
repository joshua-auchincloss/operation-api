use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use crate::{Definitions, Ident, namespace::Namespace};

pub struct Context {
    pub namespaces: BTreeMap<Ident, Namespace>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            namespaces: Default::default(),
        }
    }

    pub fn finish(&mut self) -> crate::Result<()> {
        for ns in self.namespaces.values_mut() {
            ns.check()?;
        }
        Ok(())
    }

    pub fn load_from_source(
        &mut self,
        source: PathBuf,
    ) -> crate::Result<()> {
        self.with_definition(Definitions::load_from_path(source)?)
    }

    pub fn load_from_sources(
        &mut self,
        sources: Vec<PathBuf>,
    ) -> crate::Result<()> {
        for source in sources {
            self.load_from_source(source)?;
        }
        Ok(())
    }

    pub fn get_or_create_ns(
        &mut self,
        ns: &Ident,
    ) -> &mut Namespace {
        if !self.namespaces.contains_key(&ns) {
            self.namespaces
                .insert(ns.clone(), Namespace::new(ns.clone()));
        }

        self.namespaces.get_mut(&ns).unwrap()
    }

    pub fn with_definition(
        &mut self,
        def: Definitions,
    ) -> crate::Result<()> {
        self.get_or_create_ns(def.namespace())
            .with_definition(def)
    }

    pub fn with_definitions(
        &mut self,
        defs: Vec<Definitions>,
    ) -> crate::Result<()> {
        for def in defs {
            self.with_definition(def)?;
        }
        Ok(())
    }
}
