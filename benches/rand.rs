use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::roll;

fn basic_rand(c: &mut Criterion) {
    c.bench_function("basic_rand", |b| {
        b.iter(|| {
            (0..50).into_iter().for_each(|_| {
                roll(black_box(20.0));
            })
        })
    });
}

criterion_group!(benches, basic_rand);
criterion_main!(benches);
