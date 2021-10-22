use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rollo::game::{Event, EventProcessor};

use rollo::server::tokio;
use rollo::{
    error::Error,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use std::time::Duration;

static WORLD: Lazy<MyWorld> = Lazy::new(|| {
    let world = MyWorld {
        time: AtomicI64::new(0),
        events: Mutex::new(EventProcessor::new()),
    };

    // Add an event
    world
        .events
        .lock()
        .add_event(Arc::new(MyEvent), Duration::from_secs(5));

    world
});

#[tokio::main]
async fn main() {
    let mut socket_manager = WorldSocketMgr::new(&*WORLD);

    socket_manager
        .start_game_loop(Duration::from_millis(250))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

struct MyEvent;

impl Event for MyEvent {
    fn on_execute(&self, _diff: i64) {
        println!("Event executed at {}", WORLD.time());
    }

    fn is_deletable(&self) -> bool {
        false
    }
}
// Implement WorldTime
#[rollo::world_time]
struct MyWorld {
    events: Mutex<EventProcessor<MyEvent>>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, diff: i64) {
        println!("Update at {}", WORLD.time());
        self.events.lock().update(diff);
    }
}

struct MyWorldSession {
    socket_tools: SocketTools,
}

#[rollo::async_trait]
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
