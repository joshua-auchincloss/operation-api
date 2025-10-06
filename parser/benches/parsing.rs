use criterion::{Criterion, criterion_group, criterion_main};
use operation_api_parser::{
    ast::ty::Type,
    tokens::{Parse, tokenize},
};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tokenize test message", |b| {
        b.iter(|| tokenize(black_box(include_str!("../samples/message_with_enum.pld"))).unwrap())
    });

    for (t, desc) in &[
        (
            "
            namespace foo;

            #![version(1)]

            struct Abc {
                a: i32
            };
            ",
            "struct",
        ),
        (
            "
            namespace test;

            #![version(1)]

            enum IntEnum {
                A = 1,
                B = 42,
            };
        ",
            "enum",
        ),
    ] {
        c.bench_function(&format!("ast parse '{desc}'"), |b| {
            b.iter(|| {
                let tt = operation_api_parser::ast::AstStream::from_str(black_box(t)).unwrap();
                std::hint::black_box(tt);
            })
        });
    }

    for t in &[
        "i32",
        "f32",
        "complex",
        "never",
        "oneof i32|i64",
        "i64[][]",
        "i64[]",
    ] {
        c.bench_function(&format!("type parse '{t}'"), |b| {
            let tt = tokenize(black_box(t)).unwrap();
            b.iter(|| {
                let t: Type = Type::parse(&mut std::hint::black_box(tt.clone())).unwrap();
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
