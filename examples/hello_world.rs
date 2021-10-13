use async_trait::async_trait;
use rollo::packet::to_bytes;
use rollo::tokio;
use rollo::{
    error::Error,
    packet::Packet,
    server::{
        dos_protection::DosPolicy,
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let world = Box::leak(Box::new(MyWorld {
        elapsed: AtomicI64::new(0),
    }));

    let mut socket_manager = WorldSocketMgr::new(world);
    socket_manager
        .start_game_loop(15)
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

struct MyWorld {
    elapsed: AtomicI64,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, _diff: i64) {}

    fn time(&self) -> i64 {
        self.elapsed.load(Ordering::Acquire)
    }

    fn update_time(&self, new_time: i64) {
        self.elapsed.store(new_time, Ordering::Release);
    }

    fn get_packet_limits(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        (10, 1024 * 10, DosPolicy::Log)
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
        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }
    async fn on_dos_trigger(_world_session: &Arc<Self>, _world: &'static MyWorld, _cmd: u16) {}
    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(world_session: &Arc<Self>, _world: &'static MyWorld, packet: Packet) {
        // If the message received is Login(1), send a response to the player.
        if packet.cmd == 1 {
            // Create a packet without payload
            let new_packet = to_bytes(10, None);
            if let Ok(new_packet) = new_packet {
                // Send it to the player
                world_session.socket_tools.send_data(new_packet.freeze());
            }
        }
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {
        println!("Closed");
    }
}
