use crossbeam::atomic::AtomicCell;
use std::time::Duration;

/// # Interval Manager
/// ## Usage
/// ```rust, no_run
/// use rollo::game::{IntervalExecutor, IntervalMgr};
/// use std::time::Duration;
/// use std::sync::Arc;
///
/// let interval_mgr = IntervalMgr::new(Duration::from_millis(100));
/// let battleground_mgr = Arc::new(BattlegroundMgr);
/// interval_mgr.update(100, &*battleground_mgr, Arc::clone(&battleground_mgr));
///
/// struct BattlegroundMgr;
///
/// impl IntervalExecutor for BattlegroundMgr {
///     type Container = Arc<Self>;
///    
///     fn on_update(&self, diff: i64, _container: Self::Container) {
///     }
/// }
/// ```
#[derive(Debug)]
pub struct IntervalMgr {
    current: AtomicCell<i64>,
    interval: i64,
}

impl IntervalMgr {
    /// ## Create the interval Timer
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::IntervalMgr;
    /// use std::time::Duration;
    ///
    /// let interval_timer = IntervalMgr::new(Duration::from_millis(50));
    /// ```
    pub fn new(interval: Duration) -> Self {
        Self {
            current: AtomicCell::new(0),
            interval: interval.as_millis() as i64,
        }
    }

    /// ## Update and executes if time passed
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::game::{IntervalExecutor, IntervalMgr};
    /// use std::time::Duration;
    /// use std::sync::Arc;
    ///
    /// let interval_mgr = IntervalMgr::new(Duration::from_millis(100));
    /// let battleground_mgr = Arc::new(BattlegroundMgr);
    /// interval_mgr.update(100, &*battleground_mgr, Arc::clone(&battleground_mgr));
    ///
    /// struct BattlegroundMgr;
    ///
    /// impl IntervalExecutor for BattlegroundMgr {
    ///     type Container = Arc<Self>;
    ///    
    ///     fn on_update(&self, diff: i64, _container: Self::Container) {
    ///         assert!(diff >= 50)
    ///     }
    /// }
    /// ```
    pub fn update<T>(&self, diff: i64, object: &T, container: T::Container)
    where
        T: IntervalExecutor,
    {
        if self.current.fetch_update(|f| Some(f + diff)).is_err() {
            return;
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

/// Interval Executor
pub trait IntervalExecutor {
    type Container;
    /// When event is executed.
    fn on_update(&self, _diff: i64, container: Self::Container);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update() {
        let timer = IntervalMgr::new(Duration::from_millis(50));
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
        let timer = IntervalMgr::new(Duration::from_millis(25));
        timer.current.store(30);
        timer.reset();
        assert_eq!(timer.current.load(), 5);
    }

    #[test]
    fn test_is_passed() {
        let timer = IntervalMgr::new(Duration::from_millis(25));

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

    impl IntervalExecutor for TestW {
        type Container = Option<u8>;

        fn on_update(&self, diff: i64, _container: Self::Container) {
            assert!(diff >= 50)
        }
    }
}
