#![allow(unused)]

#[cfg(any(feature = "chrono", feature = "time"))]
mod test {

    use operation_api_sdk::{Defined, module};

    include!("shared.rs");

    #[module("examples/gen-a")]
    mod test_gen {
        pub fn preserved() {}
    }

    #[test]
    fn definition_rt() {
        let def = test_gen::abc_corp_test::BasicStructWithReadme::definition();
        let ser = toml::to_string(def)
            .unwrap()
            .replace("\r", "");

        const EXPECT: &'static str = include_str!("../../samples/test-struct-readme.toml");
        assert_eq!(ser, EXPECT.replace("\r", ""));

        test_gen::preserved();
    }
}
