use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use rollo::{
    error::Error,
    packet::Packet,
    server::{
        world::World,
        world_session::{SocketTools, WorldSession},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
    time::sleep,
};

#[tokio::test]
async fn test_latency() {
    setup(6666).await;
    sleep(Duration::from_secs(1)).await;

    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    connect.write(&packet().to_vec()).await.unwrap();

    let size = connect.read_u32().await.unwrap();
    let cmd = connect.read_u16().await.unwrap();
    let date = connect.read_u64().await.unwrap();
    let latency = connect.read_i64().await.unwrap();

    assert_eq!(size, 16);
    assert_eq!(cmd, 0);
    assert_eq!(date, 1625663748);
    assert_eq!(latency, 75);
}

fn packet() -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.put_u32(16);
    bytes.put_u16(0);
    bytes.put_u64(1625663748);
    bytes.put_i64(75);

    bytes
}

async fn setup(port: u32) -> JoinHandle<()> {
    let world = Box::new(MyWorld {
        elapsed: AtomicU64::new(0),
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

struct MyWorld {
    elapsed: AtomicU64,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn time(&self) -> i64 {
        self.elapsed.load(Ordering::Acquire) as i64
    }

    fn update_time(&self, _new_time: i64) {}
}
