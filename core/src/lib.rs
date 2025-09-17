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
    ($t: path) => {
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
    Toml(#[from] toml::de::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    // internal
    #[error("{0}")]
    Generation(#[from] crate::generate::GenerationError),

    #[error("{tag} {name} already exists in {ns}")]
    NamespaceConflict {
        name: Ident,
        tag: &'static str,
        ns: Ident,
    },

    #[error("{name} is not found in {ns}")]
    NameNotFound { name: Ident, ns: Ident },
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
        CompoundType, Definitions, Field, FieldOrRef, FieldsList, Operation, Struct, Type, map,
    };

    const BASIC_STRUCT: &'static str = include_str!("../../samples/basic-struct.toml");
    const BASIC_OP: &'static str = include_str!("../../samples/basic-op.toml");

    #[test]
    fn test_de_basic_struct() {
        let namespace = crate::Ident::new("abc.corp.namespace");
        let s: Definitions = toml::from_str(&BASIC_STRUCT).unwrap();
        let s2 = Definitions::StructV1(Struct {
            name: "some_struct".into(),
            description: None,
            namespace: namespace.clone(),
            version: 2,
            fields: FieldsList::new(map!({
                a: Field::builder().ty(Type::I32).optional(false).namespace(None).name(None).build(),
                b: Field::builder().ty(Type::F32).optional(true).namespace(None).name(None).build(),
                c: Field::builder().ty(
                    ty!([[f32; 4]; 4])
                ).optional(false).namespace(None).name(None).build(),
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
                .name("add".into())
                .version(1)
                .namespace("abc.corp.namespace".into())
                .description("add a sequence of numbers together".into())
                .inputs(FieldsList::new(map!({
                    values: Field::builder().ty(ty!(Vec<u32>)).optional(false).namespace(None).name(None).build()
                })))
                .outputs(FieldsList::new(map!({
                    value: Field::builder().ty(ty!(u32)).optional(false).namespace(None).name(None).build()
                })))
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
    }
}
