use crate::server::world::World;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};
use tokio::task::yield_now;

cfg_not_precise_time! {
    use tokio::time::sleep;
}

cfg_precise_time! {
    use spin_sleep::SpinSleeper;
}

/// Main Loop with an interval
#[derive(Debug)]
pub struct GameLoop {
    date: AtomicI64,
    interval: i64,
}

impl GameLoop {
    /// Create the GameLoop with an interval.
    /// ```rust, no_run
    /// use rollo::game::GameLoop;
    /// use std::time::Duration;
    ///
    /// let game_loop = GameLoop::new(Duration::from_millis(25));
    /// ```
    pub fn new(interval: Duration) -> Self {
        Self {
            date: AtomicI64::new(Self::current_timestamp().unwrap()),
            interval: interval.as_millis() as i64,
        }
    }

    fn current_timestamp() -> Result<i64, SystemTimeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_millis() as i64)
    }

    /// Start the Game Loop
    pub async fn start(&mut self, world: &'static impl World) {
        loop {
            let last_time = self.date.load(Ordering::Acquire);
            let current = self.update_game_time();
            let diff = GameLoop::get_diff(last_time, current);

            world.update_time(current);

            World::update(world, diff);

            self.sleep_until_interval().await;

            yield_now().await;
        }
    }

    fn get_sleep_time(&self) -> i64 {
        let new_date = Self::current_timestamp();
        if let Ok(new_date) = new_date {
            let execution_diff = new_date - self.date.load(Ordering::Acquire);

            if self.interval > execution_diff {
                return self.interval - execution_diff;
            }
        }

        0
    }

    const fn get_diff(old: i64, current: i64) -> i64 {
        if old >= current {
            0
        } else {
            current - old
        }
    }

    fn update_game_time(&mut self) -> i64 {
        let current = Self::current_timestamp();

        if let Ok(current) = current {
            self.date.store(current, Ordering::Release);

            current
        } else {
            self.date.load(Ordering::Relaxed)
        }
    }

    async fn sleep_until_interval(&mut self) {
        let sleep_time = self.get_sleep_time();
        if sleep_time > 0 {
            #[cfg(all(not(test), not(feature = "precise_time")))]
            sleep(Duration::from_millis(self.get_sleep_time() as u64)).await;
            #[cfg(any(test, feature = "precise_time"))]
            SpinSleeper::default().sleep(Duration::from_millis(self.get_sleep_time() as u64));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Instant};

    use crate::server::{world::WorldTime, world_session::WorldSession};

    use super::*;
    use async_trait::async_trait;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_get_sleep_time() {
        let mut game_loop = GameLoop::new(Duration::from_millis(75));
        sleep(Duration::from_millis(500)).await;
        assert_eq!(game_loop.get_sleep_time(), 0);
        game_loop.update_game_time();
        sleep(Duration::from_millis(10)).await;
        let time = game_loop.get_sleep_time();
        assert!(time > 55 && time < 70);
        game_loop.update_game_time();
        sleep(Duration::from_millis(50)).await;
        let time = game_loop.get_sleep_time();
        assert!(time > 10 && time < 30);
        game_loop.update_game_time();
        sleep(Duration::from_millis(75)).await;
        assert_eq!(game_loop.get_sleep_time(), 0);
    }

    #[tokio::test]
    async fn test_sleep_loop() {
        let mut game_loop = GameLoop::new(Duration::from_millis(25));
        let timer = Instant::now();
        game_loop.sleep_until_interval().await;
        let sleep_time = timer.elapsed().as_millis();
        assert!((21..=30).contains(&sleep_time));

        game_loop.update_game_time();

        game_loop.sleep_until_interval().await;
        let sleep_time = timer.elapsed().as_millis();
        assert!((42..=57).contains(&sleep_time));

        game_loop.update_game_time();

        game_loop.sleep_until_interval().await;
        let sleep_time = timer.elapsed().as_millis();
        assert!((68..=85).contains(&sleep_time));
    }

    #[tokio::test]
    async fn test_update_time() {
        let mut game_loop = GameLoop::new(Duration::from_millis(25));
        sleep(Duration::from_millis(10)).await;
        let old = game_loop.date.load(Ordering::Acquire);
        let new_date = game_loop.update_game_time();
        assert!(old != new_date && new_date > old);
    }

    #[should_panic(expected = "Test : update")]
    #[tokio::test]
    async fn test_loop() {
        let mut game_loop = GameLoop::new(Duration::from_millis(25));
        let world = Box::new(TestGameLoop {
            time: AtomicI64::new(0),
        });
        let world = Box::leak(world);
        tokio::time::timeout(Duration::from_secs(1), game_loop.start(world))
            .await
            .unwrap();
    }

    #[test]
    fn test_get_diff() {
        let diff = GameLoop::get_diff(100, 150);
        assert_eq!(diff, 50);
    }

    struct SessionTest;

    #[async_trait]
    impl WorldSession<TestGameLoop> for SessionTest {
        async fn on_open(
            _socket_tools: crate::server::world_session::SocketTools,
            _world: &'static TestGameLoop,
        ) -> Result<Arc<Self>, crate::error::Error> {
            todo!()
        }

        fn socket_tools(&self) -> &crate::server::world_session::SocketTools {
            todo!()
        }

        async fn on_message(
            _world_session: &Arc<Self>,
            _world: &'static TestGameLoop,
            _packet: crate::packet::Packet,
        ) {
            todo!()
        }

        async fn on_close(_world_session: &Arc<Self>, _world: &'static TestGameLoop) {
            todo!()
        }
    }

    struct TestGameLoop {
        time: AtomicI64,
    }

    impl World for TestGameLoop {
        type WorldSessionimplementer = SessionTest;

        fn update(&'static self, _diff: i64) {
            panic!("Test : update");
        }
    }

    impl WorldTime for TestGameLoop {
        fn time(&self) -> i64 {
            self.time.load(Ordering::Acquire)
        }

        fn update_time(&self, new_time: i64) {
            let old = self.time();
            self.time.store(new_time, Ordering::Release);
            assert!(new_time > old);
            assert_ne!(10, new_time);
        }
    }
}
