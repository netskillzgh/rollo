use criterion::{criterion_group, criterion_main, Criterion};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rollo::flatbuffers_pool;

fn basic_get(c: &mut Criterion) {
    flatbuffers_pool!(1000, TEST, get_builder);
    generate_builders(1);
    c.bench_function("flat_buffers", |b| {
        b.iter(|| {
            (0..500).into_par_iter().for_each(|_| {
                get_builder();
            })
        })
    });
}

criterion_group!(benches, basic_get);
criterion_main!(benches);
