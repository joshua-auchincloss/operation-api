use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use crate::generate::RustConfig;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident(String);

impl Ident {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self::from(s)
    }
}

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

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
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

    Binary,

    CompoundType(CompoundType),
}

macro_rules! typemap {
    (
        $non_compound: expr,
        $compound: expr
    ) => {};
}

impl From<Field<Ident>> for Field<Option<Ident>> {
    fn from(value: Field<Ident>) -> Self {
        Self {
            name: Some(value.name),
            namespace: Some(value.namespace),
            ty: value.ty,
            description: value.description,
            optional: value.optional,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Field<Ns> {
    pub name: Ns,

    pub namespace: Ns,

    #[serde(rename = "type")]
    pub ty: Type,

    #[serde(default = "crate::utils::default_null")]
    pub description: Option<String>,

    #[serde(default = "crate::utils::default_no")]
    pub optional: bool,
}

impl Type {
    pub fn ty(
        &self,
        opts: &RustConfig,
    ) -> proc_macro2::TokenStream {
        match self {
            Type::Complex => todo!(),

            Type::Bool => quote::quote!(bool),

            Type::I8 => quote::quote!(i8),
            Type::I16 => quote::quote!(i8),
            Type::I32 => quote::quote!(i32),
            Type::I64 => quote::quote!(i64),

            Type::U8 => quote::quote!(u8),
            Type::U16 => quote::quote!(u8),
            Type::U32 => quote::quote!(u32),
            Type::U64 => quote::quote!(u64),

            Type::F32 => quote::quote!(f32),
            Type::F64 => quote::quote!(f64),

            Type::Binary => quote::quote!(Vec<u8>),
            Type::String => quote::quote!(String),

            Type::DateTime => quote::quote!(chrono::DateTime::<chrono::Utc>),

            Type::CompoundType(outer_ty) => {
                match outer_ty {
                    CompoundType::Array { ty } => {
                        let inner = ty.ty(opts);
                        quote::quote!(
                            Vec<#inner>
                        )
                    },
                    CompoundType::Option { ty } => {
                        let inner = ty.ty(opts);
                        quote::quote!(
                            Option<#inner>
                        )
                    },
                    CompoundType::SizedArray { size, ty } => {
                        let inner = ty.ty(opts);
                        let size = crate::generate::rust::lit(format!("{size}"));
                        quote::quote!(
                            [#inner; #size]
                        )
                    },
                    _ => todo!(),
                }
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum FieldOrRef {
    Field(Field<Option<Ident>>),
    Ref {
        #[serde(rename = "ref")]
        to: Ident,
    },
}

impl FieldOrRef {
    pub fn unwrap_field(&self) -> &Field<Option<Ident>> {
        match self {
            Self::Field(field) => field,
            _ => panic!("expected field, got ref"),
        }
    }
}

impl From<Field<Option<Ident>>> for FieldOrRef {
    fn from(value: Field<Option<Ident>>) -> Self {
        Self::Field(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct FieldsList(BTreeMap<Ident, FieldOrRef>);

impl FieldsList {
    pub fn new<M: Into<BTreeMap<Ident, FieldOrRef>>>(map: M) -> Self {
        Self(map.into())
    }
}

impl DerefMut for FieldsList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for FieldsList {
    type Target = BTreeMap<Ident, FieldOrRef>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
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
    pub fn name(&self) -> Ident {
        match self {
            Self::FieldV1(field) => field.name.clone(),
            Self::OperationV1(op) => op.name.clone(),
            Self::StructV1(def) => def.name.clone(),
        }
    }

    pub fn namespace(&self) -> &Ident {
        match self {
            Self::FieldV1(field) => &field.namespace,
            Self::OperationV1(op) => &op.namespace,
            Self::StructV1(def) => &def.namespace,
        }
    }

    pub fn load_data(
        data: Vec<u8>,
        ext: &str,
    ) -> crate::Result<Self> {
        Ok(match ext {
            "yaml" | "yml" => serde_yaml::from_slice(&data)?,
            "json" => serde_json::from_slice(&data)?,
            "toml" => toml::from_slice(&data)?,
            ext => unimplemented!("{ext} is not implemented"),
        })
    }

    pub fn load_from_path(path: PathBuf) -> crate::Result<Self> {
        Self::load_data(
            std::fs::read(&path)?,
            path.extension().unwrap().to_str().unwrap(),
        )
    }
}
