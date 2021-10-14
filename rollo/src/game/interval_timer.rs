//! ## Example
//!````
#![doc = include_str!("../../../examples/interval.rs")]
//!````
use std::time::Duration;

use crossbeam::atomic::AtomicCell;
use tracing::error;

// Interval Manager
#[derive(Debug)]
pub struct IntervalTimerMgr {
    current: AtomicCell<i64>,
    interval: i64,
}

impl IntervalTimerMgr {
    /// Create the interval Timer
    pub fn new(interval: Duration) -> Self {
        Self {
            current: AtomicCell::new(0),
            interval: interval.as_millis() as i64,
        }
    }

    /// Update and execute if time passed
    pub fn update<T>(&self, diff: i64, object: &T, container: T::Container)
    where
        T: IntervalTimerExecutor,
    {
        if self.current.fetch_update(|f| Some(f + diff)).is_err() {
            error!("Can't add update");
        }

        if !self.is_passed() {
            return;
        }

        let diff = self.current.load();

        object.on_update(diff, container);

        self.reset();
    }

    fn is_passed(&self) -> bool {
        self.current.load() >= self.interval
    }

    fn reset(&self) {
        let current = self.current.load();
        if current >= self.interval {
            while self
                .current
                .fetch_update(|x| Some(x % self.interval))
                .is_err()
            {}
        }
    }
}

/// Executed when interval passed
pub trait IntervalTimerExecutor {
    type Container;
    /// Executed when interval passed
    fn on_update(&self, _diff: i64, container: Self::Container);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update() {
        let timer = IntervalTimerMgr::new(Duration::from_millis(50));
        let bu = TestW;
        timer.update(25, &bu, None);
        assert_eq!(timer.current.load(), 25);
        timer.update(30, &bu, None);
        assert_eq!(5, timer.current.load());
        timer.update(20, &bu, None);
        assert_eq!(25, timer.current.load());
        timer.update(0, &bu, None);
        assert_eq!(25, timer.current.load());
        timer.update(15, &bu, None);
        assert_eq!(40, timer.current.load());
        timer.update(10, &bu, None);
        assert_eq!(0, timer.current.load());
    }

    #[test]
    fn test_reset() {
        let timer = IntervalTimerMgr::new(Duration::from_millis(25));
        timer.current.store(30);
        timer.reset();
        assert_eq!(timer.current.load(), 5);
    }

    #[test]
    fn test_is_passed() {
        let timer = IntervalTimerMgr::new(Duration::from_millis(25));
        timer.current.store(20);
        assert!(!timer.is_passed());
        timer.current.store(25);
        assert!(timer.is_passed());
        timer.current.store(15);
        assert!(!timer.is_passed());
        timer.current.store(0);
        assert!(!timer.is_passed());
    }

    struct TestW;

    impl IntervalTimerExecutor for TestW {
        type Container = Option<u8>;

        fn on_update(&self, diff: i64, _container: Self::Container) {
            assert!(diff >= 50)
        }
    }
}
