#![cfg(feature = "full")]
use async_trait::async_trait;
use rollo::{
    error::Error,
    game::GameTime,
    packet::Packet,
    server::{
        ListenerSecurity, SocketTools, World, WorldSession, WorldSocketConfiguration,
        WorldSocketMgr,
    },
    AtomicCell,
};
use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_game_loop() {
    let world = Box::new(MyWorld {
        counter: AtomicU16::new(0),
        game_time: AtomicCell::new(GameTime::new()),
    });
    let world = Box::leak(world);
    let mut server = WorldSocketMgr::with_configuration(world, WorldSocketConfiguration::default());
    _ = tokio::spawn(async move {
        let _ = server
            .start_game_loop(Duration::from_millis(100))
            .start_network(format!("127.0.0.1:{}", 6668), ListenerSecurity::Tcp);
    });

    sleep(Duration::from_secs(2)).await;

    assert_eq!(world.counter.load(Ordering::Relaxed), 10);
}

struct MyWorldSession {
    socket_tools: SocketTools,
}

#[async_trait]
impl WorldSession<MyWorld> for MyWorldSession {
    async fn on_open(
        tools: SocketTools,
        _world: &'static MyWorld,
    ) -> Result<std::sync::Arc<Self>, Error> {
        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &'static MyWorld, _packet: Packet) {}

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

struct MyWorld {
    counter: AtomicU16,
    game_time: AtomicCell<GameTime>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;

    fn update(&'static self, diff: i64, game_time: GameTime) {
        assert_eq!(game_time.timestamp, self.game_time.load().timestamp);
        assert_eq!(game_time.elapsed, self.game_time.load().elapsed);
        assert_eq!(game_time.system_time, self.game_time.load().system_time);

        let c = self.counter.fetch_add(1, Ordering::Relaxed) + 1;

        if c == 10 {
            assert!(!(c == 10), "test");
        }
    }

    fn game_time(&'static self) -> Option<&'static AtomicCell<GameTime>> {
        Some(&self.game_time)
    }
}
