#![allow(unused)]

use operation_api_core::namespace;
use operation_api_derives::Struct;
use operation_api_sdk::Defined;

#[derive(Struct)]
#[fields(version = 1)]
#[fields(describe(text = "Some struct"))]
pub struct BasicStruct {
    #[fields(describe(text = "field a"))]
    a: i32,
    b: Option<i32>,
    c: Vec<f32>,
    d: [[f32; 4]; 4],
}

#[derive(Struct)]
#[fields(version = 1)]
#[fields(describe(file = "../../samples/some-readme.md"))]
pub struct BasicStructWithReadme {
    #[fields(describe(text = "field a"))]
    a: i32,
}

namespace! {
    "abc.corp.test" {
        BasicStruct, BasicStructWithReadme
    }
}

fn smoke_basic<D: Defined, F: Fn(&String)>(
    out: &'static str,
    snap: F,
) {
    let ser = toml::to_string(D::definition()).unwrap();

    snap(&ser);

    std::fs::write(out, ser).unwrap();
}

#[test]
fn smoke_basic_with_readme() {
    smoke_basic::<BasicStructWithReadme, _>("../samples/test-struct-readme.toml", |ser| {
        insta::assert_snapshot!(ser)
    })
}

#[test]
fn smoke_basic_with_text() {
    smoke_basic::<BasicStruct, _>("../samples/test-struct-text.toml", |ser| {
        insta::assert_snapshot!(ser)
    })
}
