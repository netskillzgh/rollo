# Rollo

A multiplayer framework based on Rust.

- Tcp/Tls.
- Packet Manager (message command/payload).
- Game Loop (tick rate).
- Event Manager.
- Interval Manager.
- Dos protection/detection.

## Example

- Implement the world (server).

````rust,no_run
struct MyWorld {
    elapsed: AtomicU64,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(world: &Arc<Self>, _diff: u128, elapsed: u128) {
        world.elapsed.store(elapsed as u64, Ordering::Release);
    }

    fn elapsed(&self) -> u128 {
        self.elapsed.load(Ordering::Acquire) as u128
    }

    fn get_packet_limits(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        (10, 1024 * 10, DosPolicy::Log)
    }
}
````

- Implement the WorldSession (player session).

```rust,no_run
struct MyWorldSession {
    socket_tools: SocketTools,
}

#[async_trait]
impl WorldSession<MyWorld> for MyWorldSession {
    async fn on_open(tools: SocketTools) -> Result<std::sync::Arc<Self>, Error> {
        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &Arc<MyWorld>, _packet: Packet) {
        println!("Message");
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &Arc<MyWorld>) {}
}
````

- Start the server.

````rust,no_run
#[tokio::main]
async fn main() {
    let mut socket_manager = WorldSocketMgr::new(Arc::new(MyWorld {
        elapsed: AtomicU64::new(0),
    }));
    socket_manager
        .start_game_loop(15)
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}
````

## Packet composition

[Payload size(u32), Command(u16), payload(x)]