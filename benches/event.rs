use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::{Event, EventProcessor};
use std::{sync::Arc, time::Duration};

fn event_iter(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    for i in 0..1250 {
        for _ in 0..300 {
            event_processor.add_event(MyEvent, Duration::from_secs(i));
        }
    }
    c.bench_function("event_iter", |b| {
        b.iter(|| {
            event_processor.update(black_box(1000));
        });
    });
}

fn event_iter_rev(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    for i in (0..1250).rev() {
        for _ in (0..300).rev() {
            event_processor.add_event(MyEvent, Duration::from_secs(i));
        }
    }
    c.bench_function("event_iter_rev", |b| {
        b.iter(|| {
            event_processor.update(black_box(1000));
        });
    });
}

fn event_pass(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    for i in 0..1250 {
        for _ in 0..300 {
            event_processor.add_event(MyEvent, Duration::from_secs(i));
        }
    }
    c.bench_function("event_pass", |b| {
        b.iter(|| {
            event_processor.update(black_box(20000000));
        });
    });
}

struct MyEvent;

impl Event for MyEvent {
    fn on_execute(&self, _diff: i64) {}

    fn is_deletable(&self) -> bool {
        true
    }
}

criterion_group!(benches, event_iter, event_pass, event_iter_rev);
criterion_main!(benches);
