#![allow(unused)]

use operation_api_core::namespace;

include!("./shared.rs");

#[derive(operation_api_sdk::Enum)]
#[fields(version = 1)]
enum SomeEnum {
    A,
}

#[derive(operation_api_sdk::Struct)]
#[fields(version = 1)]
struct SomeStructWithEnum {
    #[fields(enm)]
    enum_value: SomeEnum,
}

namespace! {
    "abc.corp.test" {
        SomeEnum, SomeStructWithEnum,
    }
}

#[test]
fn test_some_struct_with_enum() {
    smoke_basic::<SomeStructWithEnum, _>("../samples/test-struct-with-enum.toml", |ser| {
        insta::assert_yaml_snapshot!(ser)
    })
}

#[test]
fn test_some_enum_in_struct() {
    smoke_basic::<SomeEnum, _>("../samples/test-enum-in-struct.toml", |ser| {
        insta::assert_yaml_snapshot!(ser)
    })
}
