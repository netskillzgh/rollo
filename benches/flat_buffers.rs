use criterion::{criterion_group, criterion_main, Criterion};
use rollo::flat_buffers_helpers::{generate_builders, get_builder};

fn basic_get(c: &mut Criterion) {
    generate_builders(1);
    c.bench_function("flat_buffers", |b| b.iter(get_builder));
}

criterion_group!(benches, basic_get);
criterion_main!(benches);
