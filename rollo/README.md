<div align="center"><img src="https://github.com/netskillzgh/rollo/blob/master/doc/rollo-logo.png?raw=true" height="48" width="48" alt="logo" style="height: 65px; width:65px;"/>
<h2>Rollo</h2></div>
<div align="center">
 <strong>
    A Rust-based multiplayer framework.
 </strong>
</div>

<br />

<div align="center">
  <a href="https://crates.io/crates/rollo">
    <img src="https://img.shields.io/crates/v/rollo.svg"
    alt="Crates" />
  </a>
  <a href="https://docs.rs/rollo">
    <img src="https://docs.rs/rollo/badge.svg"
    alt="Documentation" />
  </a>
   <a href="https://github.com/netskillzgh/rollo/actions/workflows/rust.yml">
    <img src="https://github.com/netskillzgh/rollo/actions/workflows/rust.yml/badge.svg"
    alt="Github Action" />
  </a>
   <a href="https://github.com/netskillzgh/rollo#license">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg"
    alt="License" />
  </a>
</div>

<br />

<hr>

## Features

- [Unity Client](https://github.com/netskillzgh/Rollo-Unity)
- TCP (with TLS support)
- Packet (command/payload)
- Game Loop (Update)
- Event Manager ([example](https://github.com/netskillzgh/rollo/blob/master/examples/event.rs))
- Interval Manager ([example](https://github.com/netskillzgh/rollo/blob/master/examples/interval.rs))
- DoS protection ([example](https://github.com/netskillzgh/rollo/blob/master/examples/dos.rs))

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
rollo = { version = "0.13.0", features = ["full"] }
```

## Example

```rust,no_run
use rollo::{
    error::Error,
    game::GameTime,
    packet::to_bytes,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
    AtomicCell,
};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    // lazy_static works as well.
    let world = Box::leak(Box::new(MyWorld {
        game_time: AtomicCell::new(GameTime::new()),
    }));

    let mut socket_manager = WorldSocketMgr::new(world);
    // Run the server and the game loop at a 15ms interval
    socket_manager
        .start_game_loop(Duration::from_millis(15))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

struct MyWorld {
    game_time: AtomicCell<GameTime>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, _diff: i64, game_time: GameTime) {
        println!("Update at : {}", game_time.timestamp);
    }

    // Your GameTime will be updated automatically. (Optional)
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

    async fn on_message(world_session: &Arc<Self>, _world: &'static MyWorld, packet: Packet) {
        // If the message received is Login(1), send a response to the player.
        if packet.cmd == 1 {
            // Create a packet without payload
            let new_packet = to_bytes(10, None);
            let new_packet = new_packet;
            // Send it to the player
            world_session.socket_tools.send_data(new_packet.into());
        }
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {
        println!("Session closed");
    }
}
```

## Packet Structure

[Payload size(u32); Command(u16); Payload]

## License

MIT license ([LICENSE-MIT](LICENSE-MIT))