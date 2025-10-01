use crate::{
    defs::{Spanned, span::Span},
    tokens::{ImplDiagnostic, error::LexingError, stream::TokenStream, tokens::SpannedToken},
};

pub trait Peek: Sized {
    fn peek(stream: &TokenStream) -> bool {
        if let Some(token) = stream.peek_unchecked() {
            Self::is(token)
        } else {
            false
        }
    }
    fn is(token: &SpannedToken) -> bool;
}

pub trait Parse: Sized {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError>;
    fn parse_spanned(stream: &mut TokenStream) -> Result<Spanned<Self>, LexingError> {
        let start = stream.cursor;
        let p = Self::parse(stream)?;
        let end = stream.cursor;
        Ok(Spanned::new(start, end, p))
    }
}

impl<T: Peek + Parse> Parse for Spanned<T> {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        stream.parse()
    }
}

impl<T: Peek + Parse> Parse for Option<T> {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        if stream.peek::<T>() {
            Ok(Some(T::parse(stream)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct RepeatedItem<T: Peek + Parse, Sep: Peek + Parse> {
    pub value: Spanned<T>,
    pub(crate) sep: Option<Spanned<Sep>>,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Repeated<T: Peek + Parse, Sep: Peek + Parse> {
    pub values: Vec<RepeatedItem<T, Sep>>,
}

impl<T: Peek + Parse, Sep: Peek + Parse> IntoIterator for Repeated<T, Sep> {
    type Item = RepeatedItem<T, Sep>;
    type IntoIter = <Vec<RepeatedItem<T, Sep>> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<T: Peek + Parse + ImplDiagnostic, Sep: Peek + Parse + Clone + ImplDiagnostic> Parse
    for Repeated<T, Sep>
{
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        let mut values = Vec::new();
        if !stream.peek::<T>() {
            return Err(LexingError::empty::<T>());
        }
        let first: Spanned<T> = stream.parse()?;
        let mut sep: Option<Spanned<Sep>> = None;
        if stream.peek::<Sep>() {
            let s: Spanned<Sep> = stream.parse()?;
            sep = Some(s);
        }
        values.push(RepeatedItem {
            value: first,
            sep: sep.clone(),
        });
        while let Some(..) = sep {
            if !stream.peek::<T>() {
                break;
            }
            let next: Spanned<T> = stream.parse()?;
            let mut next_sep: Option<Spanned<Sep>> = None;
            if stream.peek::<Sep>() {
                let s: Spanned<Sep> = stream.parse()?;
                next_sep = Some(s);
            }
            values.push(RepeatedItem {
                value: next,
                sep: next_sep.clone(),
            });
            sep = next_sep;
        }
        Ok(Self { values })
    }
}

impl<T: Parse + Peek> Parse for Vec<Spanned<T>> {
    fn parse(stream: &mut TokenStream) -> Result<Self, LexingError> {
        let mut out = vec![];
        loop {
            if !stream.peek::<T>() {
                break;
            }
            out.push(stream.parse()?);
        }
        Ok(out)
    }
}
