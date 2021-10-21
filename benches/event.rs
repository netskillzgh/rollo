use std::{sync::Arc, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::{Event, EventProcessor};

fn event(c: &mut Criterion) {
    let mut event_processor = EventProcessor::<MyEvent>::new();
    for i in 0..500 {
        for _ in 0..150 {
            event_processor.add_event(Arc::new(MyEvent), Duration::from_secs(i));
        }
    }

    c.bench_function("event", |b| {
        b.iter(|| {
            event_processor.update(black_box(10000));
        })
    });
}

struct MyEvent;

impl Event for MyEvent {
    fn on_execute(&self) {}

    fn is_deletable(&self) -> bool {
        true
    }
}

criterion_group!(benches, event);
criterion_main!(benches);
