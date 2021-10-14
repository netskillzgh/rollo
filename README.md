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
- Event Manager [example]().
- Interval Manager [example](examples/interval.rs).
- Dos protection/detection.

```toml
[dependencies]
rollo = { version = "0.1.0", features = ["full"] }
```

## Example

````rust,no_run
use async_trait::async_trait;
use rollo::packet::to_bytes;

use rollo::rollo_macros::world_time;
use rollo::tokio;
use rollo::{
    error::Error,
    packet::Packet,
    server::{
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use std::sync::{atomic::Ordering, Arc};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let world = Box::leak(Box::new(MyWorld {
        elapsed: AtomicI64::new(0),
    }));

    let mut socket_manager = WorldSocketMgr::new(world);
    // Run the server and the game loop with an interval (15ms)
    socket_manager
        .start_game_loop(Duration::from_millis(15))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

// Implement WorldTime
#[world_time]
struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, _diff: i64) {
        println!("Tick at : {}", self.time());
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
        println!("Session closed");
    }
}
````

## Packet

[Payload size(u32); Command(u16); Payload]

## License

MIT license ([LICENSE-MIT](LICENSE-MIT))