use std::{collections::BTreeMap, ops::Deref};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub struct Ident(String);

impl Ident {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self::from(s)
    }
}

impl<S: Into<String>> From<S> for Ident {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompoundType {
    Option {
        #[serde(rename = "type")]
        ty: Box<Type>,
    },
    Array {
        #[serde(rename = "type")]
        ty: Box<Type>,
    },
    SizedArray {
        size: usize,
        #[serde(rename = "type")]
        ty: Box<Type>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    U8,
    U16,
    U32,
    U64,

    I8,
    I16,
    I32,
    I64,

    F32,
    F64,

    Bool,

    DateTime,

    Complex,

    String,

    CompoundType(CompoundType),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder)]
pub struct Field<Ns> {
    #[serde(default)]
    pub name: Option<Ident>,

    pub namespace: Ns,

    #[serde(rename = "type")]
    pub ty: Type,

    #[serde(default = "crate::utils::default_null")]
    pub description: Option<String>,

    #[serde(default = "crate::utils::default_no")]
    pub optional: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum FieldOrRef {
    Field(Field<Option<Ident>>),
    Ref {
        #[serde(rename = "ref")]
        to: String,
    },
}

impl From<Field<Option<Ident>>> for FieldOrRef {
    fn from(value: Field<Option<Ident>>) -> Self {
        Self::Field(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct FieldsList(BTreeMap<String, FieldOrRef>);

impl FieldsList {
    pub fn new<M: Into<BTreeMap<String, FieldOrRef>>>(map: M) -> Self {
        Self(map.into())
    }
}

impl Deref for FieldsList {
    type Target = BTreeMap<String, FieldOrRef>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder)]
pub struct Struct {
    pub name: Ident,

    pub namespace: Ident,

    pub description: Option<String>,

    pub version: usize,
    pub fields: FieldsList,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder)]
pub struct Operation {
    pub name: Ident,

    pub namespace: Ident,

    #[serde(default)]
    pub description: Option<String>,

    pub version: usize,
    pub inputs: FieldsList,
    pub outputs: FieldsList,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Definitions {
    #[serde(rename = "field@v1", alias = "field")]
    FieldV1(Field<Ident>),

    #[serde(rename = "struct@v1", alias = "struct")]
    StructV1(Struct),

    #[serde(rename = "operation@v1", alias = "operation")]
    OperationV1(Operation),
}

impl Definitions {
    pub fn name(&self) -> Option<Ident> {
        match self {
            Self::FieldV1(field) => field.name.clone(),
            Self::OperationV1(op) => Some(op.name.clone()),
            Self::StructV1(def) => Some(def.name.clone()),
        }
    }

    pub fn namespace(&self) -> &Ident {
        match self {
            Self::FieldV1(field) => &field.namespace,
            Self::OperationV1(op) => &op.namespace,
            Self::StructV1(def) => &def.namespace,
        }
    }
}
