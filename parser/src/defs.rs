use crate::{Peek, tokens::ToTokens};

pub use span::Span;

pub mod span {
    #[derive(
        Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    pub struct Span {
        pub start: usize,
        pub end: usize,
    }

    impl Span {
        pub fn new(
            start: usize,
            end: usize,
        ) -> Self {
            Self { start, end }
        }

        pub fn len(&self) -> usize {
            self.end - self.start
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Spanned<T> {
    pub span: span::Span,
    pub value: T,
}

impl<T> Spanned<T> {
    pub fn new(
        start: usize,
        end: usize,
        value: T,
    ) -> Self {
        Self {
            value,
            span: span::Span::new(start, end),
        }
    }
    pub fn map<U>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }

    pub fn len(&self) -> usize {
        self.span.len()
    }

    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: ToTokens> ToTokens for Spanned<T> {
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        tt.write(&self.value)
    }
}

impl<T: ToTokens> ToTokens for &Spanned<T> {
    fn write(
        &self,
        tt: &mut crate::fmt::Printer,
    ) {
        tt.write(&self.value)
    }
}

impl<T: Peek> Peek for Spanned<T> {
    fn is(token: &crate::tokens::Token) -> bool {
        T::is(token)
    }

    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        T::peek(stream)
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Spanned<T> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
