use crate::{defs::Spanned, tokens::tokens};
use std::{fmt::Display, hash::Hash};

pub type Ident = tokens::IdentToken;

impl Eq for tokens::IdentToken {}

impl Hash for tokens::IdentToken {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.borrow_string().hash(state)
    }
}

impl Display for tokens::IdentToken {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.borrow_string())
    }
}

impl<S: Into<String>> From<S> for tokens::IdentToken {
    fn from(value: S) -> Self {
        Self::new(value.into())
    }
}

impl AsRef<String> for tokens::IdentToken {
    fn as_ref(&self) -> &String {
        self.borrow_string()
    }
}

impl AsRef<str> for tokens::IdentToken {
    fn as_ref(&self) -> &str {
        &self.borrow_string()
    }
}

impl tokens::IdentToken {
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
        self.borrow_string()
            .split("::")
            .collect::<Vec<_>>()
    }

    pub fn qualified_path(&self) -> Vec<Self> {
        let split = self.split();
        if split.len() > 1 {
            split
                .iter()
                .map(|s| tokens::IdentToken::new((*s).into()))
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
                .map(|s| tokens::IdentToken::new((*s).into()))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn object(&self) -> Self {
        let split = self.split();
        if split.len() > 1 {
            Self::new((*split.last().unwrap()).into())
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
        tokens::IdentToken::new(
            new_ident
                .iter()
                .map(|s| s.borrow_string().as_str())
                .collect::<Vec<_>>()
                .join("::"),
        )
    }
}

impl Spanned<Ident> {
    pub fn into_ident(&self) -> &Ident {
        &self.value
    }
}
