#![allow(unused)]

use operation_api_derives::module;

#[module(src = "samples/test-struct-text.toml")]
mod test_struct {}

#[test]
fn test() {}
