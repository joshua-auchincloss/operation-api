use crate::{Definitions, Ident};

pub trait OfNamespace {
    const NAMESPACE: &'static str;
}

#[macro_export]
macro_rules! namespace {
    ($ns: literal {
        $($t: path), + $(,)?
    }) => {
        $(
            impl $crate::namespace::OfNamespace for $t {
                const NAMESPACE: &'static str = $ns;
            }
        )*
    };
}

#[derive(serde::Deserialize, Debug)]
pub struct Namespace {
    pub name: Ident,

    #[serde(default)]
    defs: Vec<Definitions>,
}

impl Namespace {
    pub fn new<I: Into<Ident>>(name: I) -> Self {
        Self {
            name: name.into(),
            defs: vec![],
        }
    }

    pub fn with_definitions(
        &mut self,
        defs: &mut Vec<Definitions>,
    ) {
        self.defs.append(defs)
    }

    pub fn with_definition(
        &mut self,
        def: Definitions,
    ) {
        self.defs.push(def)
    }

    pub fn resolve_internal(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
