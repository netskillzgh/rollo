use indexmap::IndexMap;
use std::{collections::VecDeque, time::Duration};

/// # Event Processor
/// ## Usage
/// ```rust, no_run
/// use rollo::game::{EventProcessor, Event};
/// use std::time::Duration;
///
/// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
/// let event = MyEvent;
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
    events: IndexMap<i64, VecDeque<(i64, T)>>,
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
            events: IndexMap::new(),
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

        let mut keys_to_remove = IndexMap::new();

        self.events.retain(|time, events| {
            if m_time >= *time {
                while let Some(event) = events.pop_front() {
                    if event.1.to_abort() {
                        event.1.on_abort();
                    } else {
                        let diff = (m_time - time) + event.0;
                        event.1.on_execute(diff);

                        if !event.1.is_deletable() {
                            keys_to_remove
                                .entry(*time)
                                .or_insert(VecDeque::with_capacity(1))
                                .push_back((event.0, event.1));
                        }
                    }
                }

                false
            } else {
                true
            }
        });

        keys_to_remove.into_iter().for_each(|(_, events)| {
            events.into_iter().for_each(|(t, event)| {
                debug_assert!(!event.is_deletable());
                let new_time = m_time + t;
                self.events
                    .entry(new_time)
                    .or_default()
                    .push_back((t, event))
            });
        });
    }

    /// ## Add an event
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    /// use std::time::Duration;
    ///
    /// let mut event_processor = EventProcessor::<MyEvent>::new(1000000);
    /// let event = MyEvent;
    /// // The duration is the delay before the execution.
    /// event_processor.add_event(event, Duration::from_secs(5));
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    /// }
    /// ```
    pub fn add_event(&mut self, event: T, add_time: Duration) {
        let target_time = self.calcul_target_time(add_time.as_millis() as i64);
        if let Some(events) = self.events.get_mut(&target_time) {
            events.push_back((add_time.as_millis() as i64, event));
        } else {
            self.events.insert(
                target_time,
                VecDeque::from([(add_time.as_millis() as i64, event)]),
            );
        }
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
                .iter()
                .for_each(|events| events.1.iter().for_each(|event| event.1.on_abort()));
        }

        self.events.clear();
    }

    fn calcul_target_time(&self, add_time: i64) -> i64 {
        self.m_time + add_time
    }

    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{EventProcessor, Event};
    ///
    /// let event_processor = EventProcessor::<MyEvent>::new(1000000);
    /// assert!(event_processor.is_empty());
    ///
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn on_execute(&self, _diff: i64){}
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Events for an Event.
pub trait Event {
    /// Execute an event.
    fn on_execute(&self, _diff: i64);

    /// Is the event permanent?
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
    use std::sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    };

    #[test]
    fn test_add_event() {
        let mut event_processor = EventProcessor::<MyEventTest>::new(0);
        let event = new();
        let second_event = new();

        assert!(event_processor.is_empty());

        event_processor.add_event(event.clone(), Duration::from_millis(2500));

        assert_eq!(event_processor.events.get_index(0).unwrap().1.len(), 1);
        assert_eq!(event_processor.events.len(), 1);

        event_processor.add_event(event.clone(), Duration::from_millis(2500));

        assert_eq!(event_processor.events.get_index(0).unwrap().1.len(), 2);
        assert_eq!(event_processor.events.len(), 1);

        event_processor.add_event(event.clone(), Duration::from_millis(2600));

        assert_eq!(event_processor.events.len(), 2);

        event_processor.add_event(event, Duration::from_millis(2600));

        assert_eq!(event_processor.events.get_index(1).unwrap().1.len(), 2);
        assert_eq!(event_processor.events.len(), 2);

        event_processor.add_event(second_event, Duration::from_millis(2600));

        assert_eq!(event_processor.events.get_index(1).unwrap().1.len(), 3);
        assert_eq!(event_processor.events.len(), 2);
    }

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

        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        event_processor.add_event(second_event.clone(), Duration::from_millis(2500));
        event_processor.add_event(second_event.clone(), Duration::from_secs(3));

        event_processor.update(1);

        assert_eq!(event.data.life.load(Ordering::Acquire), 0);
        assert_eq!(second_event.data.life.load(Ordering::Acquire), 0);
        assert_eq!(event_processor.events.get_index(0).unwrap().1.len(), 3);
        assert_eq!(event_processor.events.get_index(1).unwrap().1.len(), 1);

        event_processor.update(2600);

        assert_eq!(event_processor.events.get_index(0).unwrap().1.len(), 1);
        assert_eq!(event_processor.events.len(), 1);

        event_processor.update(2);

        assert_eq!(event.data.life.load(Ordering::Acquire), 20);
        assert_eq!(second_event.data.life.load(Ordering::Acquire), 10);

        event_processor.update(3600);

        assert_eq!(event_processor.events.len(), 0);
        assert_eq!(20, event.data.life.load(Ordering::Acquire));
        assert_eq!(20, second_event.data.life.load(Ordering::Acquire));
    }

    #[test]
    fn test_not_deletable() {
        let mut event_processor = EventProcessor::new(0);
        let event = MyEventTest {
            data: Arc::new(GameData {
                life: AtomicI32::new(0),
                to_abort: AtomicBool::new(false),
                is_deletable: AtomicBool::new(false),
            }),
        };

        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        assert_eq!(event_processor.events.first().unwrap().1.len(), 1);
        event_processor.add_event(event.clone(), Duration::from_millis(2500));

        event_processor.update(10);
        assert_eq!(event.data.life.load(Ordering::Acquire), 0);

        assert_eq!(event_processor.events.len(), 1);
        assert_eq!(event_processor.events.first().unwrap().1.len(), 2);

        event_processor.update(2600);

        assert_eq!(event.data.life.load(Ordering::Acquire), 20);
        assert_eq!(event_processor.events.len(), 1);
        assert_eq!(event_processor.events.first().unwrap().1.len(), 2);

        event_processor.update(5500);

        assert_eq!(event.data.life.load(Ordering::Acquire), 40);
        assert_eq!(event_processor.events.len(), 1);
        assert_eq!(event_processor.events.first().unwrap().1.len(), 2);

        event.data.is_deletable.store(true, Ordering::Release);
        event_processor.update(8500);

        assert_eq!(event.data.life.load(Ordering::Acquire), 60);
        assert_eq!(event_processor.events.len(), 0);
    }

    #[test]
    fn test_abort() {
        let mut event_processor = EventProcessor::new(0);
        let event = MyEventTest {
            data: Arc::new(GameData {
                life: AtomicI32::new(0),
                to_abort: AtomicBool::new(true),
                is_deletable: AtomicBool::new(false),
            }),
        };

        event_processor.add_event(event.clone(), Duration::from_millis(2500));

        let event_second = MyEventTest {
            data: Arc::new(GameData {
                life: AtomicI32::new(0),
                to_abort: AtomicBool::new(true),
                is_deletable: AtomicBool::new(false),
            }),
        };

        event_processor.add_event(event_second.clone(), Duration::from_millis(26500));

        assert_eq!(event_processor.events.len(), 2);

        event_processor.update(2600);

        assert_eq!(event.data.life.load(Ordering::Acquire), 5);
        assert_eq!(event_processor.events.len(), 1);
    }

    #[test]
    fn test_remove_events() {
        let mut event_processor = EventProcessor::new(0);
        let event = new();

        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        event_processor.add_event(event, Duration::from_millis(2500));

        assert_eq!(event_processor.events.len(), 1);

        event_processor.remove_events(false);

        assert_eq!(event_processor.events.len(), 0);
    }

    #[test]
    fn test_remove_event() {
        let mut event_processor = EventProcessor::new(0);
        let event = new();
        let second_event = new();

        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        event_processor.add_event(event.clone(), Duration::from_millis(2500));
        event_processor.add_event(second_event.clone(), Duration::from_millis(2500));
        event_processor.add_event(second_event.clone(), Duration::from_secs(3));

        assert_eq!(event_processor.events.len(), 2);
        assert_eq!(event_processor.events.get(&2500).unwrap().len(), 3);

        event_processor.update(2600);

        assert_eq!(event_processor.events.len(), 1);
        assert_eq!(event_processor.events.get(&3000).unwrap().len(), 1);

        event_processor.update(3100);
        assert_eq!(event_processor.events.len(), 0);
    }

    struct GameData {
        life: AtomicI32,
        to_abort: AtomicBool,
        is_deletable: AtomicBool,
    }

    #[derive(Clone)]
    struct MyEventTest {
        data: Arc<GameData>,
    }

    fn new() -> MyEventTest {
        MyEventTest {
            data: Arc::new(GameData {
                life: AtomicI32::new(0),
                to_abort: AtomicBool::new(false),
                is_deletable: AtomicBool::new(true),
            }),
        }
    }

    impl Event for MyEventTest {
        fn to_abort(&self) -> bool {
            self.data.to_abort.load(Ordering::Acquire)
        }

        fn on_execute(&self, _diff: i64) {
            self.data.life.fetch_add(10, Ordering::SeqCst);
        }

        fn on_abort(&self) {
            self.data.life.store(5, Ordering::Release);
        }

        fn is_deletable(&self) -> bool {
            self.data.is_deletable.load(Ordering::Acquire)
        }
    }
}
