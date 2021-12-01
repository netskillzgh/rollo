use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// GameTime
#[derive(Debug, Clone, Copy)]
pub struct GameTime {
    instant: Instant,
    pub elapsed: Duration,
    pub system_time: Duration,
    pub timestamp: i64,
}

impl GameTime {
    /// New GameTime
    /// ```rust, no_run
    /// use rollo::game::GameTime;
    ///
    /// let game_time = GameTime::new();
    /// ```
    pub fn new() -> Self {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Self {
            system_time: duration,
            timestamp: duration.as_millis() as i64,
            instant: Instant::now(),
            elapsed: Duration::ZERO,
        }
    }

    pub(crate) fn update_time(&mut self) -> bool {
        let duration = Self::current_timestamp();
        self.system_time = duration;
        self.timestamp = duration.as_millis() as i64;
        self.elapsed = self.instant.elapsed();

        true
    }

    pub(crate) fn current_timestamp() -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn test_new() {
        let game_time = GameTime::new();
        assert_eq!(
            game_time.timestamp,
            game_time.system_time.as_millis() as i64
        );
        assert_eq!(game_time.elapsed, Duration::ZERO);
    }

    #[test]
    fn test_current_timestamp() {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        assert!(GameTime::current_timestamp() >= timestamp);
    }

    #[test]
    fn test_update_time() {
        let mut game_time = GameTime::new();
        let timestamp = game_time.timestamp;
        let ellapsed = game_time.elapsed;
        let system_time = game_time.system_time;
        assert_eq!(ellapsed, Duration::ZERO);
        sleep(Duration::from_millis(1));

        let r = game_time.update_time();
        assert!(r);
        assert!(game_time.timestamp > timestamp);
        assert!(game_time.elapsed > ellapsed);
        assert!(game_time.system_time > system_time);
    }
}
