use operation_api_sdk::Defined;

const EXCPECT_SOME_STRUCT: &'static str = include_str!("../../../samples/test-struct-readme.toml");

#[test]
fn test_it() {
    let def = test_gen_a::operations::abc_corp_test::BasicStructWithReadme::definition();

    let as_toml = toml::to_string(def).unwrap();
    assert_eq!(EXCPECT_SOME_STRUCT, as_toml)
}
