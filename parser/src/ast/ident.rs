use crate::{defs::Spanned, tokens::tokens};
use std::{fmt::Display, hash::Hash};

#[derive(Debug, Clone)]
pub struct Ident(tokens::IdentToken);

impl PartialEq for Ident {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0.borrow_string() == other.0.borrow_string()
    }
}

impl Eq for Ident {}

impl Hash for Ident {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.0.borrow_string().hash(state)
    }
}

impl Display for Ident {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0.borrow_string())
    }
}

impl<S: Into<String>> From<S> for Ident {
    fn from(value: S) -> Self {
        Self(tokens::IdentToken::new(value.into()))
    }
}

impl AsRef<String> for Ident {
    fn as_ref(&self) -> &String {
        self.0.borrow_string()
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        &self.0.borrow_string()
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
        self.0
            .borrow_string()
            .split("::")
            .collect::<Vec<_>>()
    }

    pub fn qualified_path(&self) -> Vec<Self> {
        let split = self.split();
        if split.len() > 1 {
            split
                .iter()
                .map(|s| Self(tokens::IdentToken::new((*s).into())))
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
                .map(|s| Self(tokens::IdentToken::new((*s).into())))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn object(&self) -> Self {
        let split = self.split();
        if split.len() > 1 {
            Self(tokens::IdentToken::new((*split.last().unwrap()).into()))
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
        Self(tokens::IdentToken::new(
            new_ident
                .iter()
                .map(|s| s.0.borrow_string().as_str())
                .collect::<Vec<_>>()
                .join("::"),
        ))
    }
}

impl Spanned<Ident> {
    pub fn into_ident(&self) -> &Ident {
        &self.value
    }
}
