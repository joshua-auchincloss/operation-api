pub mod context;
pub mod generate;
pub mod namespace;
pub mod ty;
pub(crate) mod utils;
pub use paste::paste;
pub use ty::*;

#[macro_export]
macro_rules! ty {
    (complex) => {
        $crate::Type::Complex
    };
    (datetime) => {
        $crate::Type::DateTime
    };
    (Option<$t: tt> ) => {
        $crate::Type::CompoundType($crate::CompoundType::Option {
            ty: Box::new($crate::ty! {$t}),
        })
    };
    (Vec<$t: tt>) => {
        $crate::Type::CompoundType($crate::CompoundType::Array {
            ty: Box::new($crate::ty! {$t}),
        })
    };
    ([$t: tt; $sz: literal]) => {
        $crate::Type::CompoundType($crate::CompoundType::SizedArray {
            ty: Box::new($crate::ty! {$t}),
            size: $sz,
        })
    };
    (@enm $t: path) => {
        $crate::Type::CompoundType($crate::CompoundType::Enum {
            to: stringify!($t).into(),
        })
    };
    ($t: path) => {
        // $crate::Type::Enum {
        //     $crate::
        // }
        $crate::paste! { $crate::Type::[<$t:camel>] }
    };
}

pub trait Defined: Sized + Send + Sync {
    fn definition() -> &'static Definitions;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("toml error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("{tag} {name} already exists in {ns}")]
    NamespaceConflict {
        name: Ident,
        tag: &'static str,
        ns: Ident,
    },

    #[error("{name} is not found in {ns}")]
    NameNotFound { name: Ident, ns: Ident },

    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("glob error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("[{src}] {error}")]
    SourceFile { error: Box<Self>, src: String },

    #[error("'{ident}' is not contigious with {desc}")]
    ContigiousError { ident: Ident, desc: String },
}

impl Error {
    pub fn with_source(
        self,
        src: String,
    ) -> Self {
        Self::SourceFile {
            error: Box::new(self),
            src,
        }
    }

    pub fn with_source_init(src: String) -> impl FnOnce(Self) -> Self {
        |err| err.with_source(src)
    }

    pub fn from_with_source_init<E: Into<Self>>(src: String) -> impl FnOnce(E) -> Self {
        |err| Self::with_source_init(src)(err.into())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
// #[macro_export]
// macro_rules! submit {
//     ($item: tt) => {
//         inventory::submit! {
//             $item
//         }
//     };
// }

// pub fn definitions() -> Vec<&'static Definitions> {
//     let mut v = Vec::new();
//     for it in inventory::iter::<Definitions>() {
//         v.push(it);
//     }
//     v
// }

#[cfg(test)]
mod test {

    use crate::{
        CompoundType, Definitions, Enum, Field, FieldOrRef, FieldsList, Meta, Named, Operation,
        Struct, Type, VariantKind, Version, map,
    };

    const BASIC_STRUCT: &'static str = include_str!("../../samples/basic-struct.toml");
    const BASIC_OP: &'static str = include_str!("../../samples/basic-op.toml");
    const BASIC_ENUM: &'static str = include_str!("../../samples/basic-enum.toml");

    fn empty_meta() -> Meta<Option<crate::Ident>, Option<crate::Ident>, Option<Version>> {
        Meta::builder()
            .name(None)
            .namespace(None)
            .version(None)
            .build()
    }

    #[test]
    fn test_de_basic_struct() {
        let namespace = crate::Ident::new("abc.corp.namespace");
        let s: Definitions = toml::from_str(&BASIC_STRUCT).unwrap();
        let s2 = Definitions::StructV1(Struct {
            meta: Meta {
                name: "some_struct".into(),
                description: None,
                namespace: namespace.clone(),
                version: 2_usize.into(),
            },
            fields: FieldsList::new(map!({
                a: Field::builder().ty(Type::I32).optional(false).meta(empty_meta()).build(),
                b: Field::builder().ty(Type::F32).optional(true).meta(empty_meta()).build(),
                c: Field::builder().ty(
                    ty!([[f32; 4]; 4])
                ).optional(false).meta(empty_meta()).build(),
                d: FieldOrRef::Ref{
                    to: "c".into()
                }
            })),
        });
        assert_eq!(s, s2)
    }

    #[test]
    fn test_de_basic_op() {
        let s: Definitions = toml::from_str(&BASIC_OP).unwrap();
        let expect = Definitions::OperationV1(
            Operation::builder()
            .meta(Meta::builder().name("add".into())
            .version(1_usize.into())
            .namespace("abc.corp.namespace".into())
            .description("add a sequence of numbers together".into()).build() )
                .inputs(FieldsList::new(map!({
                    values: Field::builder().ty(ty!(Vec<u32>)).optional(false).meta(empty_meta()).build()
                })))
                .outputs(FieldsList::new(map!({
                    value: Field::builder().ty(ty!(u32)).optional(false).meta(empty_meta()).build()
                })))
                .build(),
        );
        assert_eq!(s, expect);
    }

    #[test]
    fn test_de_basic_enum() {
        let s: Definitions = toml::from_str(&BASIC_ENUM).unwrap();
        let expect = Definitions::EnumV1(
            Enum::builder()
                .meta(
                    Meta::builder()
                        .name("some_enum".into())
                        .version(1_usize.into())
                        .namespace("abc.corp.namespace".into())
                        .description("some enum description".into())
                        .build(),
                )
                .variants(Named::new([
                    (
                        "A".into(),
                        VariantKind {
                            meta: Meta::builder()
                                .name("A".into())
                                .namespace(None)
                                .version(None)
                                .build(),
                            value: crate::StrOrInt::Int(1),
                        },
                    ),
                    (
                        "B".into(),
                        VariantKind {
                            meta: Meta::builder()
                                .description("Some b variant = 2".into())
                                .name("B".into())
                                .namespace(None)
                                .version(None)
                                .build(),
                            value: crate::StrOrInt::Int(2),
                        },
                    ),
                ]))
                .build(),
        );

        assert_eq!(s, expect);
    }

    #[test]
    fn ty_resolution() {
        let t = ty! {
            i32
        };

        assert_eq!(t, Type::I32);

        let t = ty! {
            Option<i32>
        };

        assert_eq!(
            t,
            Type::CompoundType(CompoundType::Option {
                ty: Box::new(Type::I32)
            })
        );

        let t = ty! {
            [i32; 4]
        };

        assert_eq!(
            t,
            Type::CompoundType(CompoundType::SizedArray {
                size: 4,
                ty: Box::new(Type::I32)
            })
        );

        let t = ty! {
            Vec<i32>
        };

        assert_eq!(
            t,
            Type::CompoundType(CompoundType::Array {
                ty: Box::new(Type::I32)
            })
        );

        #[allow(unused)]
        enum TestEnum {}

        let t = ty! {
           @enm TestEnum
        };

        assert_eq! {
            t, Type::CompoundType(CompoundType::Enum { to: "TestEnum".into() })
        };
    }
}
