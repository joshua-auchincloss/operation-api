use payloadrs_parser::parser::PayloadParser;

use std::sync::Once;

const ONCE: Once = Once::new();

fn compile() {
    ONCE.call_once(|| {
        let mut cmd = std::process::Command::new("cargo");

        cmd.args(vec!["build", "--bin", "compile"]);

        let out = cmd.output().unwrap();

        println!("Compile output: {}", String::from_utf8_lossy(&out.stdout));
        println!("Compile error: {}", String::from_utf8_lossy(&out.stderr));
    });
}

fn compile_err(src: &str) -> String {
    let mut cmd = std::process::Command::new("../target/debug/compile");
    cmd.arg(src);

    let mut out = cmd.output().unwrap();

    let mut tgt = out.stdout;
    tgt.append(&mut out.stderr);

    String::from_utf8_lossy(&tgt).into()
}

fn test_basic_compile_err(src: &str, expect: &str) {
    compile();

    let err = compile_err(src);
    std::fs::write("compile_err.txt", &err).unwrap();
    assert_eq!(err, expect);
}

macro_rules! assert_diagnostic {
    ($src: literal , $expect: literal) => {
        const EXPECT: &'static str = include_str!(concat!("../", $expect));
        test_basic_compile_err($src, EXPECT);
    };
}

#[test]
fn test_errs_with_context() {
    assert_diagnostic!(
        "samples/errors/invalid.pld",
        "samples/errors/invalid_parse.txt"
    );
}
