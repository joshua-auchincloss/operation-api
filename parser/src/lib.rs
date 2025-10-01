pub mod defs;
pub mod parser;

pub mod ctx;
pub(crate) mod utils;

#[cfg(test)]
pub(crate) mod tst;

use std::{
    convert::Infallible,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use crate::{defs::Ident, parser::Rule};
use thiserror::Error;

pub use ctx::Parse;
pub use ctx::parse_files;

#[derive(Debug, Error)]
pub enum Error {
    // #[error("{0}")]
    // Miette(#[from] miette::Error),
    #[error("{0}")]
    RuleViolation(#[from] pest::error::Error<Rule>),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("definition error for {rule:#?}: {src}")]
    DefError {
        src: String,
        rule: crate::parser::Rule,
    },

    #[error("definition error for {rules:#?}: {src}")]
    DefsError {
        src: String,
        rules: Vec<crate::parser::Rule>,
    },

    #[error("parse int error: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("parse bool error: {0}")]
    ParseBool(#[from] ParseBoolError),
    #[error("parse float error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{namespace:#?} has conflicts. {ident} is declared multiple times.")]
    IdentConflict { namespace: Vec<Ident>, ident: Ident },

    #[error("namespace is not declared")]
    NsNotDeclared,

    #[error("only one namespace may be declared in a payload declaration file.")]
    NsConflict,

    #[error("resolution error. could not resolve {ident}")]
    ResolutionError { ident: Ident },

    #[error("value error: {value} is not valid for types {tys:#?}")]
    ValueError {
        value: String,
        tys: Vec<crate::defs::Type>,
    },
}

impl Error {
    pub fn def<T>(rule: crate::parser::Rule) -> Self {
        let ty = std::any::type_name::<T>();
        Self::DefError {
            src: ty.into(),
            rule,
        }
    }

    pub fn defs<T, I: IntoIterator<Item = crate::parser::Rule>>(rules: I) -> Self {
        let ty = std::any::type_name::<T>();
        Self::DefsError {
            src: ty.into(),
            rules: rules.into_iter().collect(),
        }
    }

    pub fn conflict(namespace: Vec<Ident>, ident: Ident) -> Self {
        Self::IdentConflict { namespace, ident }
    }

    pub fn resolution(ident: Ident) -> Self {
        Self::ResolutionError { ident }
    }

    pub fn value_error<I: IntoIterator<Item = crate::defs::Type>>(value: String, tys: I) -> Self {
        Self::ValueError {
            value,
            tys: tys.into_iter().collect(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// macro_rules! std {
//     ($($mod: ident), + $(,)?) => {
//         pub mod payloads_std {
//             paste::paste!{
//                 $(
//                     pub const [<$mod: upper>]: &'static str = include_str!(
//                         concat!("../../std/", stringify!($mod), ".pld")
//                     );
//                 )*


//                 pub fn get_std() -> miette::Result<crate::ctx::Ctx> {
//                     use crate::Parse;

//                     let ctx = crate::ctx::Ctx::new();
//                     $(
//                         ctx.parse_data(
//                             concat!("std/", stringify!($mod), ".pld"),
//                             [<$mod: upper>],
//                         )?;
//                     )*

//                     Ok(ctx)
//                 }
//             }

//         }
//     };
// }

// std! {
//     iso,
// }
