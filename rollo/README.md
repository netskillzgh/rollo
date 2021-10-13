# Rollo

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](
https://github.com/netskillzgh/rollo#license)
[![Cargo](https://img.shields.io/crates/v/rollo.svg)](
https://crates.io/crates/rollo)
[![Documentation](https://docs.rs/rollo/badge.svg)](
https://docs.rs/rollo)

A multiplayer framework based on Rust.

- Tcp (support Tls).
- Packet Manager (message command/payload).
- Game Loop (tick rate).
- Event Manager.
- Interval Manager.
- Dos protection/detection.

```toml
[dependencies]
rollo = { version = "0.1.0", features = ["full"] }
```

## Example

````rust,no_run
use rollo::{
    async_trait,
    server::{
        dos_protection::DosPolicy,
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
    tokio,
};
use rollo::packet::to_bytes;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

#[tokio::main]
async fn main() {
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
    async fn on_dos_trigger(_world_session: &Arc<Self>, _world: &'static MyWorld, _cmd: u16) {}

    async fn on_open(
        socket_tools: rollo::server::world_session::SocketTools,
        _world: &'static MyWorld,
    ) -> Result<std::sync::Arc<Self>, rollo::error::Error> {
        Ok(Arc::new(Self { socket_tools }))
    }

    fn socket_tools(&self) -> &rollo::server::world_session::SocketTools {
        &self.socket_tools
    }

     async fn on_message(world_session: &Arc<Self>, _world: &'static MyWorld, _packet: Packet) {
        // Create the packet without payload
        let packet = to_bytes(10, None);
        if let Ok(packet) = packet {
            // Send it to the player
            world_session.socket_tools.send_data(packet.freeze());
        }
    }

    async fn on_close(_world_session: &std::sync::Arc<Self>, _world: &'static MyWorld) {}
}
````

## Packet composition

[Payload size(u32), Command(u16), payload(x)]

## License

MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)