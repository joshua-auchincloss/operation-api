pub use operation_api_core::{
    CompoundType, Defined, Definitions, Enum, Field, FieldsList, Meta, Named, OneOf, OneOfVariant,
    Operation, StrOrInt, Struct, Type, Typed, VariantKind, Version, map, namespace,
    namespace::OfNamespace,
};
pub use operation_api_derives::{Enum, OneOf, Struct, module};
pub use serde_repr::{Deserialize_repr as IntDeserialize, Serialize_repr as IntSerialize};
