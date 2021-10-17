use rollo::server::async_trait;

use rollo::server::dos_protection::DosPolicy;
use rollo::server::rollo_macros::world_time;
use rollo::server::tokio;
use rollo::{
    error::Error,
    packet::Packet,
    server::{
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let world = Box::leak(Box::new(MyWorld {
        elapsed: AtomicI64::new(0),
    }));

    let mut socket_manager = WorldSocketMgr::new(world);
    socket_manager
        .start_game_loop(Duration::from_millis(15))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

#[world_time]
struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;

    fn get_packet_limit(&self, cmd: u16) -> (u16, u32, DosPolicy) {
        match cmd {
            // Max 10 requests /sec, Size max 1024 and close the session if exceed the limit.
            1 => (10, 1024, DosPolicy::Close),
            _ => (20, 1024, DosPolicy::Log),
        }
    }
}

struct MyWorldSession {
    socket_tools: SocketTools,
}

#[async_trait]
impl WorldSession<MyWorld> for MyWorldSession {
    async fn on_dos_attack(_world_session: &Arc<Self>, _world: &'static MyWorld, cmd: u16) {
        println!("DoS attack detected for cmd {}.", cmd);
    }

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
