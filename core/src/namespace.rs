use std::collections::BTreeMap;

use convert_case::Casing;

use crate::{Definitions, Field, FieldOrRef, Ident, Operation, Struct, generate::LanguageTrait};

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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Namespace {
    pub name: Ident,

    pub fields: BTreeMap<Ident, Field<Ident>>,
    pub ops: BTreeMap<Ident, Operation>,
    pub defs: BTreeMap<Ident, Struct>,
}

impl Namespace {
    pub fn new<I: Into<Ident>>(name: I) -> Self {
        Self {
            name: name.into(),
            fields: Default::default(),
            ops: Default::default(),
            defs: Default::default(),
        }
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

    pub fn with_definition(
        &mut self,
        def: Definitions,
    ) -> crate::Result<()> {
        match def {
            Definitions::FieldV1(field) => {
                unique_ns_def(
                    &mut self.fields,
                    field.name.clone(),
                    &self.name,
                    field,
                    "field",
                )?;
            },
            Definitions::StructV1(def) => {
                unique_ns_def(&mut self.defs, def.name.clone(), &self.name, def, "struct")?;
            },
            Definitions::OperationV1(op) => {
                unique_ns_def(&mut self.ops, op.name.clone(), &self.name, op, "operation")?;
            },
        }

        Ok(())
    }

    pub fn resolve_field(
        &self,
        name: &Ident,
    ) -> crate::Result<&Field<Ident>> {
        self.fields.get(name).ok_or_else(|| {
            crate::Error::NameNotFound {
                name: name.clone(),
                ns: self.name.clone(),
            }
        })
    }

    pub fn resolve_field_types(&mut self) -> crate::Result<()> {
        for (def_name, def) in self.defs.clone().iter() {
            let mut swap = vec![];
            for (field_name, field) in &*def.fields {
                match field {
                    FieldOrRef::Field(..) => {},
                    FieldOrRef::Ref { to } => {
                        if let Some(local_ref) = def.fields.get(&to) {
                            swap.push((field_name.clone(), local_ref.clone()));
                        } else {
                            swap.push((
                                field_name.clone(),
                                FieldOrRef::Field(self.resolve_field(to)?.clone().into()),
                            ));
                        }
                    },
                }
            }

            for (name, field) in swap {
                self.defs
                    .get_mut(&def_name)
                    .unwrap()
                    .fields
                    .insert(name, field);
            }
        }

        Ok(())
    }

    pub fn check(&mut self) -> crate::Result<()> {
        self.resolve_field_types()?;
        Ok(())
    }

    pub fn normalized_path<L: LanguageTrait>(&self) -> String {
        self.name
            .to_string()
            .replace(".", "_")
            .to_case(L::file_case())
    }
}

#[inline]
fn unique_ns_def<T>(
    sources: &mut BTreeMap<Ident, T>,
    name: Ident,
    ns: &Ident,
    def: T,
    tag: &'static str,
) -> crate::Result<()> {
    match sources.insert(name.clone(), def) {
        Some(..) => {
            Err(crate::Error::NamespaceConflict {
                ns: ns.clone(),
                name,
                tag,
            })
        },
        None => Ok(()),
    }
}
