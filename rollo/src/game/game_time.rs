use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Default)]
pub struct GameTime {
    pub system_time: Duration,
    pub ms_time: i64,
    pub timestamp: i64,
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            system_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            ms_time: Self::current_timestamp().unwrap(),
            timestamp: Self::current_timestamp().unwrap(),
        }
    }

    pub(crate) fn update_time(&mut self) {
        self.system_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        self.ms_time = 1;
        self.timestamp = Self::current_timestamp().unwrap();
    }

    pub(crate) fn current_timestamp() -> Result<i64, SystemTimeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_millis() as i64)
    }
}
