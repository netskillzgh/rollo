use crossbeam::atomic::AtomicCell;
use tracing::error;

pub struct IntervalTimerMgr {
    current: AtomicCell<i64>,
    interval: i64,
}

impl IntervalTimerMgr {
    /// Create the interval Timer
    pub fn new(interval: i64) -> Self {
        Self {
            current: AtomicCell::new(0),
            interval,
        }
    }

    /// Update and execute if time passed
    pub fn update<T>(&self, diff: i64, object: &T)
    where
        T: IntervalTimerExecutor,
    {
        if self.current.fetch_update(|f| Some(f + diff)).is_err() {
            error!("Can't add update");
        }

        if !self.is_passed() {
            return;
        }

        object.on_update(diff);

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

pub trait IntervalTimerExecutor {
    /// Executed when interval passed
    fn on_update(&self, _diff: i64) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update() {
        let timer = IntervalTimerMgr::new(25);
        let bu = X;
        timer.update(25, &bu);
        assert_eq!(timer.current.load(), 0);
        timer.update(30, &bu);
        assert_eq!(5, timer.current.load());
        timer.update(20, &bu);
        assert_eq!(0, timer.current.load());
        timer.update(0, &bu);
        assert_eq!(0, timer.current.load());
        timer.update(15, &bu);
        assert_eq!(15, timer.current.load());
        timer.update(10, &bu);
        assert_eq!(0, timer.current.load());
    }

    #[test]
    fn test_reset() {
        let timer = IntervalTimerMgr::new(25);
        timer.current.store(30);
        timer.reset();
        assert_eq!(timer.current.load(), 5);
    }

    #[test]
    fn test_is_passed() {
        let timer = IntervalTimerMgr::new(25);
        timer.current.store(20);
        assert!(!timer.is_passed());
        timer.current.store(25);
        assert!(timer.is_passed());
        timer.current.store(15);
        assert!(!timer.is_passed());
        timer.current.store(0);
        assert!(!timer.is_passed());
    }

    struct X;

    impl IntervalTimerExecutor for X {}
}