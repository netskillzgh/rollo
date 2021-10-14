use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use rollo::{
    error::Error,
    packet::Packet,
    server::{
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use rollo_macros::world_time;
use tokio::{io::AsyncReadExt, net::TcpStream, task::JoinHandle, time::sleep};

#[tokio::test]
async fn connect_server() {
    setup(6666).await;

    sleep(Duration::from_secs(1)).await;

    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    let size = connect.read_u32().await.unwrap();
    let cmd = connect.read_u16().await.unwrap();
    let payload = connect.read_u16().await.unwrap();

    assert_eq!(size, 2);
    assert_eq!(cmd, 10);
    assert_eq!(payload, 25);
}

async fn setup(port: u32) -> JoinHandle<()> {
    let world = Box::new(MyWorld {
        elapsed: AtomicI64::new(0),
    });
    let world = Box::leak(world);
    let mut server = WorldSocketMgr::new(world);

    tokio::spawn(async move {
        server
            .start_network(format!("127.0.0.1:{}", port), ListenerSecurity::Tcp)
            .await
            .unwrap();
    })
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
        assert_eq!(tools.id, 1);

        let age: u16 = 25;
        tools.send(10, Some(&age.to_be_bytes()));

        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &'static MyWorld, _packet: Packet) {}

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}

    async fn on_dos_trigger(_world_session: &Arc<Self>, _world: &'static MyWorld, _cmd: u16) {}
}

#[world_time]
struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
}
