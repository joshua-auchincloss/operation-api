use crate::tokens::{AstResult, LexingError};
use std::fmt::{Display, Formatter};

pub(crate) const VALID_FORMS: &str =
    "`schema::ns`, `pkg::ns::Thing`, `ns::Thing`, `pkg::ns`, `external::pkg::Obj`";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Path {
    /// `schema::ns`
    LocalNamespace { namespace: String },
    /// `schema::ns::Thing`
    LocalObject { namespace: String, object: String },
    /// `ns::Thing` or `pkg::ns`
    Ambiguous { first: String, second: String },
    /// `external::pkg::Obj`
    ExternalObject {
        schema: String,
        namespace: String,
        object: String,
    },
}

impl Path {
    pub fn parse(s: &str) -> AstResult<Self> {
        Ok(match s.split("::").collect::<Vec<_>>().as_slice() {
            ["schema", ns] => {
                Self::LocalNamespace {
                    namespace: ns.to_string(),
                }
            },
            ["schema", ns, obj] => {
                Self::LocalObject {
                    namespace: ns.to_string(),
                    object: obj.to_string(),
                }
            },
            [schema, ns, obj] if *schema != "schema" => {
                Self::ExternalObject {
                    schema: schema.to_string(),
                    namespace: ns.to_string(),
                    object: obj.to_string(),
                }
            },
            [first, second] if *first != "schema" => {
                Self::Ambiguous {
                    first: first.to_string(),
                    second: second.to_string(),
                }
            },
            _ => {
                return Err(LexingError::InvalidPath {
                    input: s.into(),
                    reason: format!("expected one of {VALID_FORMS}"),
                });
            },
        })
    }
}

impl Display for Path {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::LocalNamespace { namespace } => write!(f, "schema::{namespace}"),
            Self::LocalObject { namespace, object } => write!(f, "schema::{namespace}::{object}"),
            Self::ExternalObject {
                schema,
                namespace,
                object,
            } => {
                write!(f, "{schema}::{namespace}::{object}")
            },
            Self::Ambiguous { first, second } => {
                write!(f, "{first}::{second}")
            },
        }
    }
}

mod serde {

    impl serde::Serialize for super::Path {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer, {
            serializer.serialize_str(&format!("{self}"))
        }
    }

    struct Visitor;

    impl<'v> serde::de::Visitor<'v> for Visitor {
        type Value = super::Path;

        fn expecting(
            &self,
            f: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            write!(f, "string")
        }

        fn visit_str<E>(
            self,
            v: &str,
        ) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
            super::Path::parse(v).map_err(|err| serde::de::Error::custom(format!("{err}")))
        }
    }

    impl<'de> serde::Deserialize<'de> for super::Path {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>, {
            deserializer.deserialize_str(Visitor)
        }
    }
}
