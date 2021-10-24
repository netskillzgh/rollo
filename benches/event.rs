use std::{sync::Arc, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::{Event, EventProcessor};

fn event_iter(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    for i in 0..1250 {
        for _ in 0..300 {
            event_processor.add_event(Arc::new(MyEvent), Duration::from_secs(i));
        }
    }

    c.bench_function("event_iter", |b| {
        b.iter(|| {
            event_processor.update(black_box(1000));
        })
    });
}

fn event_pass(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    for i in 0..1250 {
        for _ in 0..300 {
            event_processor.add_event(Arc::new(MyEvent), Duration::from_secs(i));
        }
    }

    c.bench_function("event_pass", |b| {
        b.iter(|| {
            event_processor.update(black_box(2000000));
        })
    });
}

struct MyEvent;

impl Event for MyEvent {
    fn on_execute(&self, _diff: i64) {}

    fn is_deletable(&self) -> bool {
        true
    }
}

criterion_group!(benches, event_iter, event_pass);
criterion_main!(benches);
