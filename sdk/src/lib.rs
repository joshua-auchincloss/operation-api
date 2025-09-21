pub use operation_api_core::{
    CompoundType, Defined, Definitions, Enum, Field, FieldsList, Meta, Named, OneOf, OneOfVariant,
    Operation, Struct, Type, Typed, Version, map, namespace::OfNamespace,
};
pub use operation_api_derives::{Enum, OneOf, Struct, module};
pub use serde_repr::{Deserialize_repr as IntDeserialize, Serialize_repr as IntSerialize};
