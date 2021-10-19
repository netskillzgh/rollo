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
- Event Manager - [example](https://github.com/netskillzgh/rollo/blob/master/examples/event.rs).
- Interval Manager - [example](https://github.com/netskillzgh/rollo/blob/master/examples/interval.rs).
- Dos protection - [example](https://github.com/netskillzgh/rollo/blob/master/examples/dos.rs).
- flatbuffers - [example](https://github.com/netskillzgh/rollo/blob/master/examples/flatbuffers.rs)

```toml
[dependencies]
rollo = { version = "0.4.0", features = ["full"] }
```

## Example

```rust,no_run
use rollo::packet::to_bytes;
use rollo::server::async_trait;
use rollo::server::rollo_macros::world_time;
use rollo::server::tokio;
use rollo::{
    error::Error,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let world = Box::leak(Box::new(MyWorld {
        time: AtomicI64::new(0),
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
```

## Packet

[Payload size(u32); Command(u16); Payload]

## License

MIT license ([LICENSE-MIT](LICENSE-MIT))