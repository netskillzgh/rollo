use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rollo::{
    error::Error,
    game::{Event, EventProcessor, GameTime},
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
    tokio, AtomicCell,
};
use std::{sync::Arc, time::Duration};

static WORLD: Lazy<MyWorld> = Lazy::new(|| {
    let game_time = GameTime::new();
    let world = MyWorld {
        events: Mutex::new(EventProcessor::new(game_time.timestamp)),
        game_time: AtomicCell::new(game_time),
    };

    // Add an event
    world
        .events
        .lock()
        .add_event(MyEvent, Duration::from_secs(5));

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
        println!("Event executed at {}", WORLD.game_time.load().timestamp);
    }

    fn is_deletable(&self) -> bool {
        false
    }
}

struct MyWorld {
    events: Mutex<EventProcessor<MyEvent>>,
    game_time: AtomicCell<GameTime>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, _diff: i64, game_time: GameTime) {
        self.events.lock().update(game_time.timestamp);
    }

    fn game_time(&'static self) -> Option<&'static AtomicCell<GameTime>> {
        Some(&self.game_time)
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
