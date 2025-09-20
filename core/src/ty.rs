use std::{
    collections::BTreeMap,
    fmt::Display,
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use convert_case::Casing;

use crate::{generate::RustConfig, namespace::Namespace};

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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(usize);

impl Version {
    pub const fn new(version: usize) -> Self {
        Self(version)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self(1)
    }
}

impl<S: Into<usize>> From<S> for Version {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

pub trait Contigious<T> {
    fn is_contigious(
        &self,
        parent: &Ident,
    ) -> crate::Result<T>;
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
    Enum {
        #[serde(rename = "ref")]
        to: Ident,
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

impl From<Field<Ident>> for Field<Option<Ident>> {
    fn from(value: Field<Ident>) -> Self {
        Self {
            meta: Meta {
                name: Some(value.meta.name),
                namespace: Some(value.meta.namespace),
                description: value.meta.description,
                version: value.meta.version,
            },
            ty: value.ty,
            optional: value.optional,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Field<Ns> {
    #[serde(flatten)]
    pub meta: Meta<Ns, Ns, Option<Version>>,

    #[serde(rename = "type")]
    pub ty: Type,

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
                    CompoundType::Enum { to } => {
                        let as_rs_ref = super::generate::rust::ident(&to.to_string());
                        quote::quote!(#as_rs_ref)
                    },
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
                }
            },
        }
    }

    pub fn rust_attrs(&self) -> proc_macro2::TokenStream {
        match self {
            Self::CompoundType(ty) => {
                match ty {
                    CompoundType::Enum { .. } => quote::quote!(#[fields(enm)]),
                    _ => quote::quote! {},
                }
            },
            _ => quote::quote! {},
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Reffable<T> {
    Value(T),
    Ref {
        #[serde(rename = "ref")]
        to: Ident,
    },
}

pub type FieldOrRef = Reffable<Field<Option<Ident>>>;
pub type EnumOrRef = Reffable<Enum>;

impl<T> Reffable<T> {
    pub fn unwrap_value(&self) -> &T {
        match self {
            Self::Value(v) => v,
            _ => panic!("expected value, got ref"),
        }
    }
}
impl<T> From<T> for Reffable<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Named<T>(BTreeMap<Ident, T>);

pub type FieldsList = Named<FieldOrRef>;

impl<T> Named<T> {
    pub fn new<M: Into<BTreeMap<Ident, T>>>(map: M) -> Self {
        Self(map.into())
    }
}

impl<T> DerefMut for Named<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for Named<T> {
    type Target = BTreeMap<Ident, T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged, rename_all = "snake_case")]
pub enum StrOrInt {
    String(String),
    Int(usize),
}

#[derive(PartialEq, Debug)]
pub enum EnumValueType {
    String,
    Int,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct VariantKind {
    #[serde(flatten)]
    pub meta: Meta<Ident, Option<Ident>, Option<Version>>,
    pub value: StrOrInt,
}

impl Named<VariantKind> {
    fn ty_of(
        &self,
        it: &VariantKind,
    ) -> EnumValueType {
        match &it.value {
            StrOrInt::Int(_) => EnumValueType::Int,
            StrOrInt::String(_) => EnumValueType::String,
        }
    }
}

impl Contigious<EnumValueType> for Named<VariantKind> {
    fn is_contigious(
        &self,
        parent: &Ident,
    ) -> crate::Result<EnumValueType> {
        let outer_ty = self.ty_of(self.0.values().next().unwrap());
        for (ident, variant) in &self.0 {
            let ty = self.ty_of(variant);
            if outer_ty != ty {
                return Err(crate::Error::ContigiousError {
                    ident: format!("{parent}::{ident}").into(),
                    desc: format!("expected {outer_ty:#?}, received {ty:#?}"),
                });
            }
        }
        Ok(outer_ty)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Meta<IdentTy, NsTy, Version> {
    pub name: IdentTy,
    pub namespace: NsTy,

    #[serde(default)]
    pub description: Option<String>,

    pub version: Version,
}

impl<IdentTy, NsTy, Version> Meta<IdentTy, NsTy, Version> {
    pub fn doc_comment(&self) -> proc_macro2::TokenStream {
        crate::generate::rust::comment(&self.description)
    }

    pub fn op_comment(&self) -> proc_macro2::TokenStream {
        match &self.description {
            Some(desc) => {
                quote::quote!(
                    #[fields(
                        describe(text = #desc)
                    )]
                )
            },
            None => quote::quote!(),
        }
    }
}

impl<IdentTy, Ns> Meta<IdentTy, Ns, Version> {
    pub fn version(&self) -> proc_macro2::TokenStream {
        crate::generate::rust::lit(self.version.0.to_string())
    }
}

impl<IdentTy: ToString, Ns, Version> Meta<IdentTy, Ns, Version> {
    pub fn ident_as_pascal(&self) -> syn::Ident {
        crate::generate::rust::ident(
            self.name
                .to_string()
                .to_case(convert_case::Case::Pascal),
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Struct {
    #[serde(flatten)]
    pub meta: Meta<Ident, Ident, Version>,
    pub fields: FieldsList,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Operation {
    #[serde(flatten)]
    pub meta: Meta<Ident, Ident, Version>,

    pub inputs: FieldsList,
    pub outputs: FieldsList,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, bon::Builder, Clone)]
pub struct Enum {
    #[serde(flatten)]
    pub meta: Meta<Ident, Ident, Version>,

    pub variants: Named<VariantKind>,
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

    #[serde(rename = "enum@v1", alias = "enum")]
    EnumV1(Enum),

    #[serde(rename = "namespace@v1", alias = "namespace")]
    NamespaceV1(Namespace),
}

impl Definitions {
    // pub fn meta(&self) -> &Meta<Ident, usize> {
    //     match self {
    //         Self::FieldV1(field) => &field.meta,
    //         Self::OperationV1(op) => &op.meta,
    //         Self::StructV1(def) => &def.meta,
    //         Self::EnumV1(enm) => &enm.meta,
    //     }
    // }

    pub fn name(&self) -> &Ident {
        match self {
            Self::FieldV1(v) => &v.meta.name,
            Self::StructV1(v) => &v.meta.name,
            Self::OperationV1(v) => &v.meta.name,
            Self::EnumV1(v) => &v.meta.name,
            Self::NamespaceV1(ns) => &ns.name,
        }
    }

    pub fn namespace(&self) -> &Ident {
        match self {
            Self::FieldV1(v) => &v.meta.namespace,
            Self::StructV1(v) => &v.meta.namespace,
            Self::OperationV1(v) => &v.meta.namespace,
            Self::EnumV1(v) => &v.meta.namespace,
            Self::NamespaceV1(ns) => &ns.name,
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

    pub fn write_data<W: Write>(
        &self,
        w: &mut W,
        ext: &str,
    ) -> crate::Result<()> {
        Ok(match ext {
            "yaml" | "yml" => serde_yaml::to_writer(w, self)?,
            "json" => serde_json::to_writer(w, self)?,
            "toml" => {
                w.write(toml::to_string(self)?.as_bytes())?;
                w.flush()?;
            },
            ext => unimplemented!("{ext} is not implemented"),
        })
    }

    pub fn load_from_path(path: PathBuf) -> crate::Result<Self> {
        Self::load_data(
            std::fs::read(&path)?,
            path.extension().unwrap().to_str().unwrap(),
        )
        .map_err(crate::Error::from_with_source_init(
            path.display().to_string(),
        ))
    }

    pub fn export(
        &self,
        path: PathBuf,
    ) -> crate::Result<()> {
        self.write_data(
            &mut std::fs::File::create(&path)?,
            path.extension().unwrap().to_str().unwrap(),
        )
    }
}
