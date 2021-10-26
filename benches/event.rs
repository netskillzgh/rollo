use std::{sync::Arc, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollo::game::{Event, EventProcessor};

fn event_iter(c: &mut Criterion) {
    c.bench_function("event_iter", |b| {
        let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
        for i in 0..1250 {
            for _ in 0..300 {
                event_processor.add_event(Arc::new(MyEvent), Duration::from_secs(i));
            }
        }
        b.iter(|| {
            event_processor.update(black_box(1000));
        });
        assert!(!event_processor.is_empty());
    });
}

fn event_pass(c: &mut Criterion) {
    c.bench_function("event_pass", |b| {
        let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
        for i in 0..1250 {
            for _ in 0..300 {
                event_processor.add_event(Arc::new(MyEvent), Duration::from_secs(i));
            }
        }
        b.iter(|| {
            event_processor.update(black_box(20000000));
        });
        assert!(event_processor.is_empty());
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
