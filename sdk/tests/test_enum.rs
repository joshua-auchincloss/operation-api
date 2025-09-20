#![allow(unused)]

use operation_api_core::namespace;
use operation_api_sdk::*;

include!("./shared.rs");

#[derive(operation_api_sdk::Enum, serde::Serialize)]
#[fields(version = 1)]
pub enum BasicStrEnum {
    #[fields(str_value = "a")]
    A,
    #[fields(str_value = "b")]
    B,
}

#[derive(operation_api_sdk::Enum, serde::Serialize)]
#[fields(version = 1, describe(text = "some int based enum"))]
pub enum BasicIntEnum {
    A,
    #[fields(describe(text = "B is 99"))]
    B = 99,
}

namespace! {
    "abc.corp.test" {
        BasicStrEnum, BasicIntEnum,
    }
}

#[test]
fn test_basic_str_enum() {
    smoke_basic::<BasicStrEnum, _>("../samples/test-str-enum.toml", |ser| {
        insta::assert_yaml_snapshot!(ser)
    })
}

#[test]
fn test_basic_int_enum() {
    smoke_basic::<BasicIntEnum, _>("../samples/test-enum.toml", |ser| {
        insta::assert_yaml_snapshot!(ser)
    })
}
