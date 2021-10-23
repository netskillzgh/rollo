use std::time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub struct GameTime {
    instant: Instant,
    pub elapsed: Duration,
    pub system_time: Duration,
    pub timestamp: i64,
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            system_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            timestamp: Self::current_timestamp().unwrap().as_millis() as i64,
            instant: Instant::now(),
            elapsed: Duration::ZERO,
        }
    }

    pub(crate) fn update_time(&mut self) {
        if let Ok(duration) = Self::current_timestamp() {
            self.system_time = duration;
            self.timestamp = duration.as_millis() as i64;
            self.elapsed = self.instant.elapsed();
        }
    }

    pub(crate) fn current_timestamp() -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(UNIX_EPOCH)
    }
}
