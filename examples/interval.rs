use rollo::server::async_trait;
use rollo::server::rollo_macros::world_time;
use rollo::server::tokio;
use rollo::server::ListenerSecurity;
use rollo::server::SocketTools;
use rollo::server::World;
use rollo::server::WorldSession;
use rollo::server::WorldSocketMgr;
use rollo::{
    error::Error,
    game::{IntervalExecutor, IntervalMgr},
    packet::Packet,
};
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let world = Box::new(MyWorld {
        time: AtomicI64::new(0),
        bg: Arc::new(BattlegroundManager {
            timer: IntervalMgr::new(Duration::from_secs(2)),
            name: String::from("Battleground 1"),
        }),
    });
    let world = Box::leak(world);

    let mut socket_manager = WorldSocketMgr::new(world);
    socket_manager
        .start_game_loop(Duration::from_millis(250))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

#[world_time]
struct MyWorld {
    bg: Arc<BattlegroundManager>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, diff: i64) {
        self.bg.timer.update(diff, &*self.bg, Arc::clone(&self.bg));
    }
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
        println!("On Open");
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

struct BattlegroundManager {
    timer: IntervalMgr,
    name: String,
}

impl IntervalExecutor for BattlegroundManager {
    type Container = Arc<Self>;
    fn on_update(&self, diff: i64, _container: Self::Container) {
        println!("Executed : {} : The diff is {}", diff, self.name);
    }
}
