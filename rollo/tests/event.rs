#![cfg(feature = "full")]
use rollo::game::{Event, EventProcessor};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::Duration;

#[test]
fn test_event_processor() {
    let mut event_processor = EventProcessor::<MyEvent>::new(0);
    let first_event = Arc::new(MyEvent {
        executed: AtomicBool::new(false),
    });
    event_processor.add_event(Arc::clone(&first_event), Duration::from_secs(10));
    let second_event = Arc::new(MyEvent {
        executed: AtomicBool::new(false),
    });
    event_processor.add_event(Arc::clone(&second_event), Duration::from_secs(15));

    // First Event
    event_processor.update(9000);
    assert!(!first_event.executed.load(Ordering::SeqCst));

    event_processor.update(10000);
    assert!(first_event.executed.load(Ordering::SeqCst));

    event_processor.update(14000);
    assert!(!second_event.executed.load(Ordering::SeqCst));

    event_processor.update(14999);
    assert!(!second_event.executed.load(Ordering::SeqCst));
    assert!(!event_processor.is_empty());

    event_processor.update(15001);
    assert!(second_event.executed.load(Ordering::SeqCst));
    assert!(event_processor.is_empty());
}

struct MyEvent {
    executed: AtomicBool,
}

impl Event for MyEvent {
    fn on_execute(&self, _diff: i64) {
        self.executed.store(true, Ordering::SeqCst);
    }
}
