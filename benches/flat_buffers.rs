use criterion::{criterion_group, criterion_main, Criterion};
use rollo::flatbuffers_pool;

fn basic_get(c: &mut Criterion) {
    flatbuffers_pool!(100, TEST, get_builder);
    generate_builders(1);
    c.bench_function("flat_buffers", |b| b.iter(get_builder));
}

criterion_group!(benches, basic_get);
criterion_main!(benches);
