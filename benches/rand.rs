use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rollo::game::roll;

fn basic_rand(c: &mut Criterion) {
    c.bench_function("basic_rand", |b| {
        b.iter(|| {
            (0..200).into_par_iter().for_each(|_| {
                roll(black_box(20.0));
            })
        })
    });
}

criterion_group!(benches, basic_rand);
criterion_main!(benches);
