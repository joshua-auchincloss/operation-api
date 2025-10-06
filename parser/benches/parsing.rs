use criterion::{Criterion, criterion_group, criterion_main};
use operation_api_parser::{
    ast::{self, ty::Type},
    tokens::{Parse, tokenize},
};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tokenize test message", |b| {
        b.iter(|| tokenize(black_box(include_str!("../samples/message_with_enum.pld"))).unwrap())
    });

    for t in &[
        "i32",
        "f32",
        "complex",
        "never",
        "oneof i32|i64",
        "i64[][]",
        "i64[]",
    ] {
        c.bench_function(&format!("parse '{t}'"), |b| {
            b.iter(|| {
                let mut tt = tokenize(black_box(t)).unwrap();
                let t: Type = Type::parse(&mut tt).unwrap();
                std::hint::black_box(t);
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
