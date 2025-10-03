use crate::defs::*;

use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);
impl Display for Ident {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S: Into<String>> From<S> for Ident {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Ident {
    pub fn path_equals(
        this: &Vec<Self>,
        other: &Vec<Self>,
    ) -> bool {
        for (i, pat) in this.iter().enumerate() {
            match other.get(i) {
                Some(other_pat) if pat == other_pat => {},
                _ => return false,
            }
        }
        true
    }

    fn split(&self) -> Vec<&str> {
        self.0.split("::").collect::<Vec<_>>()
    }

    pub fn qualified_path(&self) -> Vec<Self> {
        let split = self.split();
        if split.len() > 1 {
            split
                .iter()
                .map(|s| Self((*s).into()))
                .collect()
        } else {
            vec![self.clone()]
        }
    }

    pub fn namespace(&self) -> Vec<Self> {
        let split = self.split();
        if split.len() > 1 {
            split[..split.len() - 1]
                .iter()
                .map(|s| Self((*s).into()))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn object(&self) -> Self {
        let split = self.split();
        if split.len() > 1 {
            Self((*split.last().unwrap()).into())
        } else {
            self.clone()
        }
    }

    #[allow(clippy::ptr_arg)]
    pub fn set_namespace(
        self,
        namespace: &Vec<Self>,
    ) -> Self {
        let mut new_ident = namespace.clone();
        new_ident.push(self.object());
        Self(
            new_ident
                .iter()
                .map(|s| s.0.as_str())
                .collect::<Vec<_>>()
                .join("::"),
        )
    }
}

impl Spanned<Ident> {
    pub fn into_ident(self) -> Ident {
        self.value
    }
}
