use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::rand::roll;

fn basic_rand(c: &mut Criterion) {
    c.bench_function("basic_rand", |b| b.iter(|| roll(black_box(20.0))));
}

criterion_group!(benches, basic_rand);
criterion_main!(benches);
