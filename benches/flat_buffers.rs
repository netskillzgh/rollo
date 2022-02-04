use criterion::{criterion_group, criterion_main, Criterion};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn basic_get(c: &mut Criterion) {
    c.bench_function("flat_buffers", |b| {
        b.iter(|| {
            (0..500).into_par_iter().for_each(|_| {
                rollo::flatbuffers_helpers::FLAT_BUFFER_BUILDER_GENERATOR.create();
            })
        })
    });
}

criterion_group!(benches, basic_get);
criterion_main!(benches);
