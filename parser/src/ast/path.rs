use crate::tokens::{AstResult, LexingError};
use std::fmt::{Display, Formatter};

pub(crate) const VALID_FORMS: &str = "`schema::a::b`, `pkg::a::b`, `a::b`, `single`";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Path {
    Local { bits: Vec<String> },
    Ambiguous { bits: Vec<String> },
}

impl Path {
    pub fn parse(s: &str) -> AstResult<Self> {
        let parts: Vec<&str> = s.split("::").collect();

        if parts.is_empty() {
            return Err(LexingError::InvalidPath {
                input: s.into(),
                reason: format!("expected one of {VALID_FORMS}"),
            });
        }

        if parts[0] == "schema" {
            let bits = parts
                .iter()
                .skip(1)
                .map(|p| p.to_string())
                .collect::<Vec<_>>();
            if bits.is_empty() {
                return Err(LexingError::InvalidPath {
                    input: s.into(),
                    reason: format!("expected one of {VALID_FORMS}"),
                });
            }
            Ok(Path::Local { bits })
        } else {
            let bits = parts
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>();
            Ok(Path::Ambiguous { bits })
        }
    }

    pub fn segments(&self) -> &Vec<String> {
        match self {
            Path::Local { bits } => bits,
            Path::Ambiguous { bits } => bits,
        }
    }
}

impl Display for Path {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Path::Local { bits } => write!(f, "{}", bits.join("::")),
            Path::Ambiguous { bits } => {
                write!(f, "{}", bits.join("::"))
            },
        }
    }
}

mod serde {
    use super::Path;

    impl serde::Serialize for Path {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer, {
            serializer.serialize_str(&format!("{}", self))
        }
    }

    struct Visitor;

    impl<'v> serde::de::Visitor<'v> for Visitor {
        type Value = Path;

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
            Path::parse(v).map_err(|err| serde::de::Error::custom(format!("{err}")))
        }
    }

    impl<'de> serde::Deserialize<'de> for Path {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>, {
            deserializer.deserialize_str(Visitor)
        }
    }
}
