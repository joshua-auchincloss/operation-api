use crate::defs::*;

use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder)]
pub struct Arg {
    pub comment: String,

    pub ident: Ident,
    pub ty: Type,

    pub raw_value: Option<String>,
    pub default_value: Option<Value>,
    pub required: bool,
}

impl FromInner for Arg {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut ty = None;
        let mut default_value = None;
        let mut required = true;
        let mut raw_value = None;
        let mut comment = String::new();

        tracing::trace!("pairs: {pairs:#?}");
        for pair in pairs {
            tracing::trace!("arg: {pair:#?}");
            match pair.as_rule() {
                Rule::arg => return Self::from_inner(pair.into_inner()),
                Rule::name => ident = Some(Ident::from_inner(Pairs::single(pair))?),
                Rule::typ => ty = Some(Type::from_inner(Pairs::single(pair))?),
                Rule::eq_value => {
                    if let Some(ref ty) = ty {
                        let inner = pair.into_inner();
                        raw_value = Some(clean_rawvalue(inner.as_str()));
                        default_value = Some(Value::from_inner(inner, ty.clone())?);
                    } else {
                        return Err(crate::Error::defs::<Self, _>([Rule::typ]));
                    }
                },
                Rule::field_sep => {
                    let inner = pair.into_inner().next();
                    match inner.map(|p| p.as_rule()) {
                        Some(Rule::optional_field_sep) => required = false,
                        _ => required = true,
                    }
                },
                Rule::optional_field_sep => {
                    required = false;
                },
                Rule::singleline_comment | Rule::multiline_comment | Rule::comment => {
                    tracing::info!("comment (arg): {pair:#?}");
                    comment += &take_comment(Pairs::single(pair));
                },
                rule => panic!("{rule:#?}"),
            }
        }

        if ident.is_none() || ty.is_none() {
            if ident.is_none() {
                tracing::error!("ident is none");
            } else {
                tracing::error!("ty is none")
            }
            Err(crate::Error::defs::<Self, _>([Rule::ident, Rule::typ]))
        } else {
            Ok(Self {
                comment,
                raw_value,
                ident: ident.unwrap(),
                ty: ty.unwrap(),
                default_value,
                required,
            })
        }
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub struct MessageDef {
    pub comment: String,
    pub ident: Ident,
    pub types: Vec<Arg>,
}

impl FromInner for MessageDef {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut types = vec![];

        let mut comment = String::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_def => return Self::from_inner(Pairs::single(pair)),
                Rule::ident | Rule::name => ident = Some(Ident::from_inner(Pairs::single(pair))?),
                Rule::arg_list => {
                    for arg in pair.into_inner() {
                        let mut comment = String::new();
                        match arg.as_rule() {
                            Rule::multiline_comment => {
                                tracing::trace!("comment before: {arg:#?}");

                                comment += &take_comment(Pairs::single(arg));
                            },
                            Rule::singleline_comment => {
                                tracing::trace!("comment after: {arg:#?}");

                                comment += &take_comment(Pairs::single(arg));
                            },
                            _ => {
                                let mut arg = Arg::from_inner(Pairs::single(arg))?;
                                arg.comment += &comment;
                                types.push(arg)
                            },
                        }
                    }
                },
                Rule::comment => {
                    comment += &take_comment(Pairs::single(pair));
                },
                rule => panic!("rule: {rule:#?}"),
            }
        }
        if ident.is_none() {
            return Err(crate::Error::def::<Self>(Rule::ident));
        }
        Ok(Self {
            comment,
            ident: ident.unwrap(),
            types,
        })
    }
}

impl Commentable for MessageDef {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}
