use std::{fmt::Display, future::pending, path::PathBuf};

use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

use crate::defs::*;

#[derive(Debug, Clone)]
pub enum PayloadTypes {
    Namespace(NamespaceDef),
    Type(TypeDef),
    Message(MessageDef),
    Import(ImportDef),
    Enum(EnumDef),
}

impl PayloadTypes {
    pub fn unwrap_namespace(self) -> NamespaceDef {
        match self {
            Self::Namespace(def) => def,
            _ => panic!("expected namespace"),
        }
    }

    pub fn unwrap_type(self) -> TypeDef {
        match self {
            Self::Type(def) => def,
            _ => panic!("expected type"),
        }
    }

    pub fn unwrap_message(self) -> MessageDef {
        match self {
            Self::Message(def) => def,
            _ => panic!("expected message"),
        }
    }

    pub fn unwrap_import(self) -> ImportDef {
        match self {
            Self::Import(def) => def,
            _ => panic!("expected import"),
        }
    }

    pub fn unwrap_enum(self) -> EnumDef {
        match self {
            Self::Enum(def) => def,
            _ => panic!("expected enum"),
        }
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub struct Payload {
    pub(crate) source: PathBuf,
    pub(crate) defs: Vec<PayloadTypes>,
}

impl Payload {
    pub fn build(
        source: PathBuf,
        pairs: Pairs<crate::parser::Rule>,
    ) -> crate::Result<Self> {
        let mut defs = vec![];
        let mut pending_comment: Option<String> = None;
        for pair in pairs {
            let rule = pair.as_rule();
            tracing::trace!("rule: {rule:#?}");
            match rule {
                Rule::payloads => return Self::build(source, pair.into_inner()),
                Rule::type_def => {
                    tracing::trace!("found singular_typedef");
                    let mut ty = TypeDef::from_inner(pair.into_inner())?;
                    apply_pending_if_forward(&mut ty, &mut pending_comment);

                    tracing::trace!("type {ty:#?}");
                    defs.push(PayloadTypes::Type(ty));
                },
                Rule::struct_def => {
                    tracing::trace!("found message");
                    let mut msg = MessageDef::from_inner(pair.into_inner())?;
                    apply_pending_if_forward(&mut msg, &mut pending_comment);

                    tracing::trace!("message {msg:#?}");
                    defs.push(PayloadTypes::Message(msg));
                },
                Rule::import_def => {
                    tracing::trace!("found import");
                    let mut import = ImportDef::from_inner(pair.into_inner())?;
                    apply_pending_if_forward(&mut import, &mut pending_comment);

                    tracing::trace!("import {import:#?}");
                    defs.push(PayloadTypes::Import(import));
                },
                Rule::namespace_def => {
                    tracing::trace!("found namespace");
                    let mut ns = NamespaceDef::from_inner(pair.into_inner())?;
                    apply_pending_if_forward(&mut ns, &mut pending_comment);

                    tracing::trace!("namespace {ns:#?}");
                    defs.push(PayloadTypes::Namespace(ns));
                },
                Rule::enum_def => {
                    tracing::trace!("found enum");
                    let mut enum_def = EnumDef::from_inner(pair.into_inner())?;
                    apply_pending_if_forward(&mut enum_def, &mut pending_comment);

                    tracing::trace!("enum {enum_def:#?}");
                    defs.push(PayloadTypes::Enum(enum_def));
                },
                Rule::multiline_comment | Rule::comment => {
                    tracing::trace!("found multiline comment");
                    pending_comment = Some(take_comment(Pairs::single(pair.clone())));
                },
                Rule::singleline_comment => {
                    tracing::trace!("found single line comment");
                    let comment = take_comment(Pairs::single(pair.clone()));
                    tracing::info!("comment: '{:#?}' '{comment:#?}'", pair.as_str());

                    // if comment.is_empty() {
                    //     continue;
                    // }

                    let last = if defs.is_empty() {
                        None
                    } else {
                        let sz = defs.len() - 1;
                        defs.get_mut(sz)
                    };

                    match last {
                        Some(last) => {
                            match last {
                                PayloadTypes::Import(import) => {
                                    tracing::trace!(
                                        "commenting on import: '{comment}' -> {import:#?}"
                                    );

                                    import.comment(comment);
                                },
                                PayloadTypes::Message(msg) => {
                                    tracing::trace!("commenting on msg: '{comment}' -> {msg:#?}");

                                    msg.comment(comment);
                                },
                                PayloadTypes::Namespace(ns) => {
                                    tracing::trace!("commenting on ns: '{comment}' -> {ns:#?}");

                                    ns.comment(comment);
                                },
                                PayloadTypes::Type(ty) => {
                                    tracing::trace!("commenting on ns: '{comment}' -> {ty:#?}");

                                    ty.comment(comment);
                                },
                                PayloadTypes::Enum(enm) => {
                                    tracing::trace!("commenting on enum: '{comment}' -> {enm:#?}");

                                    enm.comment(comment);
                                },
                            }
                        },
                        None => {
                            pending_comment = Some(comment);
                        },
                    }
                },
                Rule::inner_meta => {
                    panic!("inner meta")
                },
                Rule::EOI => {
                    break;
                },
                rule => {
                    tracing::error!("unhandled rule: {rule:#?}");
                    return Err(crate::Error::defs::<Self, _>([
                        Rule::import_def,
                        Rule::struct_def,
                        Rule::type_def,
                        Rule::payloads,
                    ]));
                },
            }
        }
        tracing::trace!("done");
        Ok(Self::builder()
            .source(source)
            .defs(defs)
            .build())
    }
}
