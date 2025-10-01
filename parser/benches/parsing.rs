use criterion::{Criterion, criterion_group, criterion_main};
use operation_api_parser::parser::PayloadParser;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse test_message", |b| {
        b.iter(|| {
            PayloadParser::parse_data(
                "dummy.pld",
                black_box(include_str!("../samples/message_with_enum.pld")),
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
