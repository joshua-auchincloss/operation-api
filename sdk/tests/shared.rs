fn smoke_basic<D: operation_api_sdk::Defined, F: Fn(&String)>(
    out: &'static str,
    snap: F,
) {
    use operation_api_sdk::Defined;

    let ser = toml::to_string(D::definition()).unwrap();

    snap(&ser);

    std::fs::write(out, ser).unwrap();
}
