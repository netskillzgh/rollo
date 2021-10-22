use criterion::{criterion_group, criterion_main, Criterion};

use std::time::Duration;

fn time_duration(c: &mut Criterion) {
    c.bench_function("time", |b| {
        let duration = Duration::from_secs(100);
        let second_duration = Duration::from_secs(100);
        b.iter(|| duration >= second_duration)
    });
}

fn time_millis(c: &mut Criterion) {
    c.bench_function("millis", |b| {
        let duration = 100000;
        let second_duration = 100000;
        b.iter(|| duration >= second_duration)
    });
}

criterion_group!(benches, time_duration, time_millis);
criterion_main!(benches);
