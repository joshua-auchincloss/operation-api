use operation_api_parser::{
    defs::{Payload, PayloadTypesSealed},
    parser::PayloadParser,
};
use std::path::PathBuf;
use test_case::test_case;

fn parse_single(contents: &str) -> miette::Result<Payload> {
    let path = PathBuf::from("test_version.pld");
    PayloadParser::parse_data(path, contents)
}

#[test_case(r#"
namespace foo;
#![version(2)]

#[version(1)]
struct Foo { a: i32 };"# , Some(2), 1 ; "outer_overrides_inner")]
#[test_case(r#"#![version(7)]
namespace foo;
struct Foo { a: i32 };"#, Some(7), 7 ; "inner_used_when_no_outer")]
#[test_case(r#"#[version(9)]
namespace foo;

#[version(9)]
struct Foo { a: i32 };"#, Some(9), 9 ; "identical_outer_and_struct")]

fn version_success_cases(
    src: &str,
    expected: Option<usize>,
    item_version: usize,
) {
    let payload = parse_single(src).expect("parse ok");
    assert_eq!(payload.version(), expected, "source:\n{src}");
    let inner = payload
        .defs
        .iter()
        .filter(|it| matches!(it, PayloadTypesSealed::Struct(..)))
        .next()
        .unwrap();
    let ver = inner.version().unwrap();
    assert_eq!(item_version, *ver, "expected version {item_version}")
}

#[test_case(r#"
namespace foo;
#[version(5)]
struct Foo { a: i32 };
#[version(6)]
#[version(7)]
struct Bar { a: i32 };"#, "version attribute conflict" ; "definition_conflict")]
#[test_case(r#"namespace foo;
struct Foo { a: i32 };"#, "version" ; "no_versions_present")]
fn version_error_cases(
    src: &str,
    expected_substr: &str,
) {
    let err = parse_single(src).expect_err("expected error");
    let msg = format!("{err}");
    assert!(
        msg.contains(expected_substr),
        "expected '{expected_substr}' in '{msg}'\nsource:\n{src}"
    );
}
