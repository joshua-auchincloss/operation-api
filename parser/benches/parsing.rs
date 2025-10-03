use criterion::{Criterion, criterion_group, criterion_main};
use operation_api_parser::{
    ast,
    tokens::{Parse, tokenize},
};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tokenize test message", |b| {
        b.iter(|| tokenize(black_box(include_str!("../samples/message_with_enum.pld"))).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
