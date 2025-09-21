use operation_api_sdk::Definitions;

#[allow(unused)]
fn smoke_basic<D: operation_api_sdk::Defined, F: Fn(&'static Definitions)>(
    out: &'static str,
    snap: F,
) {
    use operation_api_sdk::Defined;

    operation_api_testing::insta_test!(|| {
        snap(D::definition());
    });

    let ser = toml::to_string(D::definition()).unwrap();
    std::fs::write(out, ser).unwrap();
}
