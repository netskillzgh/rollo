#![cfg(feature = "full")]
use async_trait::async_trait;
use rollo::{
    error::Error,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use std::sync::{
    atomic::{AtomicI64, AtomicU16, Ordering},
    Arc,
};
use tokio::time::Duration;

#[tokio::test]
async fn test_game_loop() {
    let world = Box::new(MyWorld {
        counter: AtomicU16::new(0),
        time: AtomicI64::new(0),
    });
    let world = Box::leak(world);
    let mut server = WorldSocketMgr::new(world);
    let t = tokio::spawn(async move {
        let _ = server
            .start_game_loop(Duration::from_millis(100))
            .start_network(format!("127.0.0.1:{}", 6666), ListenerSecurity::Tcp);
    });

    let _ = t.await;

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

#[rollo::world_time]
struct MyWorld {
    counter: AtomicU16,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;

    fn update(&'static self, diff: i64) {
        assert!(self.time() != 0);
        let c = self.counter.fetch_add(1, Ordering::Relaxed) + 1;

        // First diff is 0
        if c != 1 {
            assert!((93..107).contains(&diff));
        }

        if c == 10 {
            panic!("test");
        }
    }
}
