use divan::black_box;
use operation_api_parser::{
    ast::{self, ty::Type},
    tokens::{Parse, tokenize},
};

const BENCH_STRUCT: &str = include_str!("../samples/bench_struct.pld");
const BENCH_ENUM: &str = include_str!("../samples/bench_enum.pld");
const SAMPLE_MSG: &str = include_str!("../samples/abc-corp/schema/foo/message_with_enum.pld");

#[inline(always)]
fn ast_parse(s: &str) {
    black_box(ast::AstStream::from_string(black_box(s)).unwrap());
}

#[divan::bench(name = "tokenize test message")]
fn tokenize_test_message() {
    black_box(tokenize(black_box(SAMPLE_MSG)).unwrap());
}

#[divan::bench(name = "ast parse struct")]
fn ast_parse_struct() {
    ast_parse(black_box(BENCH_STRUCT))
}

#[divan::bench(name = "ast parse enum")]
fn ast_parse_enum() {
    ast_parse(black_box(BENCH_ENUM))
}

#[divan::bench(name = "ast parse nested ast enum", args = [
    BENCH_ENUM,
    BENCH_STRUCT,
])]
fn ast_parse_nested_ast(
    bencher: divan::Bencher,
    ast: &str,
) {
    let ast = format!(
        "
    namespace abc_corp;

    #[version(1)]
    namespace foo {{
        {ast}
    }};
    "
    );
    bencher.bench(move || ast_parse(black_box(&ast)));
}

#[divan::bench(args = [
    "i32",
    "f32",
    "complex",
    "never",
    "oneof i32|i64",
    "i64[][]",
    "i64[]",
    "{ a: i32, b: i64 }",
])]
fn type_parse(t: &str) {
    let tt = tokenize(black_box(t)).unwrap();
    let mut tt_clone = tt.clone();
    let parsed: Type = Type::parse(&mut tt_clone).unwrap();
    divan::black_box(parsed);
}

fn main() {
    divan::main();
}
