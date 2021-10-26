use multimap::MultiMap;
use std::{collections::HashMap, sync::Arc, time::Duration};

/// # Event Processor
/// ## Usage
/// ```rust, no_run
/// use rollo::game::{EventProcessor, Event};
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
/// let event = MyEvent;
/// let event = Arc::new(event);
/// // The duration is the delay before the execution.
/// event_processor.add_event(event, Duration::from_secs(5));
/// event_processor.update(1005000);
///
/// struct MyEvent;
///
/// impl Event for MyEvent {
///     fn on_execute(&self, _diff: i64){}
/// }
/// ```
#[derive(Default, Debug)]
pub struct EventProcessor<T>
where
    T: Event,
{
    events: MultiMap<i64, (i64, Arc<T>)>,
    m_time: i64,
}

impl<T> EventProcessor<T>
where
    T: Event,
{
    /// ## Create an event processor
    /// Time (current time in milliseconds)
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    ///
    /// let event_processor = EventProcessor::<MyEvent>::new(1000000);
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    /// }
    /// ```
    pub fn new(time: i64) -> Self {
        Self {
            m_time: time,
            events: MultiMap::new(),
        }
    }

    /// ## Update events
    /// Time (current time in milliseconds)
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    ///
    /// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    /// // 1000000 is the time.
    /// event_processor.update(1000000);
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    /// }
    /// ```
    pub fn update(&mut self, time: i64) {
        self.m_time = time;
        let m_time = self.m_time;

        let mut keys_to_remove = HashMap::new();

        for (time, events_i) in self.events.iter_all() {
            if *time > m_time {
                continue;
            }

            keys_to_remove.insert(*time, Vec::new());

            if let Some(retain) = keys_to_remove.get_mut(&time.clone()) {
                for event in events_i.iter() {
                    if event.1.to_abort() {
                        event.1.on_abort();
                    } else {
                        let diff = (m_time - time) + event.0;
                        event.1.on_execute(diff);

                        if !event.1.is_deletable() {
                            retain.push((event.0, Arc::clone(&event.1)));
                        }
                    }
                }
            }
        }

        keys_to_remove.iter().for_each(|(key, events)| {
            self.events.remove(key);

            events.iter().for_each(|event| {
                let new_time = m_time + event.0;
                self.events.insert(new_time, event.clone());
            });
        });
    }

    /// ## Add an event
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    /// let event = MyEvent;
    /// let event = Arc::new(event);
    /// // The duration is the delay before the execution.
    /// event_processor.add_event(event, Duration::from_secs(5));
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    /// }
    /// ```
    pub fn add_event(&mut self, event: Arc<T>, add_time: Duration) {
        let target_time = self.calcul_target_time(add_time.as_millis() as i64);

        self.events
            .insert(target_time, (add_time.as_millis() as i64, event));
    }

    /// ## Remove Events
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    ///
    /// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    /// // Remove all events and abort them (on_abort()).
    /// event_processor.remove_events(true);
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    ///     fn on_abort(&self) {}
    /// }
    /// ```
    pub fn remove_events(&mut self, abort: bool) {
        if abort {
            self.events
                .iter_all()
                .for_each(|events| events.1.iter().for_each(|event| event.1.on_abort()));
        }

        self.events.clear();
    }

    fn calcul_target_time(&self, add_time: i64) -> i64 {
        self.m_time + add_time
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Events for an Event.
pub trait Event {
    /// Execute an event.
    fn on_execute(&self, _diff: i64);

    /// Event lifetime
    fn is_deletable(&self) -> bool {
        true
    }

    /// Abort at next update
    fn to_abort(&self) -> bool {
        false
    }

    /// Event aborted
    fn on_abort(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    #[test]
    fn test_calcul_time_success() {
        let mut event_processor = EventProcessor::<MyEventTest>::new(0);
        event_processor.m_time = 10;
        let result = event_processor.calcul_target_time(10);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_update_success() {
        let mut event_processor = EventProcessor::new(0);
        let event = new();
        let second_event = new();

        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&second_event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&second_event), Duration::from_secs(3));

        assert_eq!(event.life.load(Ordering::Acquire), 0);
        assert_eq!(second_event.life.load(Ordering::Acquire), 0);

        event_processor.update(2600);

        assert_eq!(event.life.load(Ordering::Acquire), 20);
        assert_eq!(second_event.life.load(Ordering::Acquire), 10);

        event_processor.update(3600);

        assert_eq!(20, event.life.load(Ordering::Acquire));
        assert_eq!(20, second_event.life.load(Ordering::Acquire));
    }

    #[test]
    fn test_not_deletable() {
        let mut event_processor = EventProcessor::new(0);
        let event = MyEventTest {
            life: AtomicI32::new(0),
            to_abort: AtomicBool::new(false),
            is_deletable: AtomicBool::new(false),
        };
        let event = Arc::new(event);

        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));

        assert_eq!(event_processor.events.len(), 1);

        event_processor.update(2600);

        assert_eq!(event_processor.events.len(), 1);
    }

    #[test]
    fn test_abort() {
        let mut event_processor = EventProcessor::new(0);
        let event = MyEventTest {
            life: AtomicI32::new(0),
            to_abort: AtomicBool::new(true),
            is_deletable: AtomicBool::new(false),
        };
        let event = Arc::new(event);

        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));

        assert_eq!(event_processor.events.len(), 1);

        event_processor.update(2600);

        assert_eq!(event.life.load(Ordering::Acquire), 5);
        assert_eq!(event_processor.events.len(), 0);
    }

    #[test]
    fn test_remove_events() {
        let mut event_processor = EventProcessor::new(0);
        let event = new();

        {
            event_processor.add_event(event, Duration::from_millis(2500));
        }

        assert_eq!(event_processor.events.len(), 1);

        {
            event_processor.remove_events(false);
        }

        assert_eq!(event_processor.events.len(), 0);
    }

    #[test]
    fn test_remove_event() {
        let mut event_processor = EventProcessor::new(0);
        let event = new();
        let second_event = new();

        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&second_event), Duration::from_millis(2500));
        event_processor.add_event(Arc::clone(&second_event), Duration::from_secs(3));

        assert_eq!(event_processor.events.len(), 2);

        assert_eq!(event_processor.events.get_vec(&2500).unwrap().len(), 3);

        event_processor.update(2600);
        assert_eq!(event_processor.events.len(), 1);
        assert_eq!(event_processor.events.get_vec(&3000).unwrap().len(), 1);

        event_processor.update(3100);

        assert_eq!(event_processor.events.len(), 0);
    }

    struct MyEventTest {
        life: AtomicI32,
        to_abort: AtomicBool,
        is_deletable: AtomicBool,
    }

    fn new() -> Arc<MyEventTest> {
        Arc::new(MyEventTest {
            life: AtomicI32::new(0),
            to_abort: AtomicBool::new(false),
            is_deletable: AtomicBool::new(true),
        })
    }

    impl Event for MyEventTest {
        fn to_abort(&self) -> bool {
            self.to_abort.load(Ordering::Acquire)
        }

        fn on_execute(&self, _diff: i64) {
            self.life.fetch_add(10, Ordering::SeqCst);
        }

        fn on_abort(&self) {
            self.life.store(5, Ordering::Release);
        }

        fn is_deletable(&self) -> bool {
            self.is_deletable.load(Ordering::Acquire)
        }
    }
}
