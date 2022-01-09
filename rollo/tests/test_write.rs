#![cfg(feature = "full")]
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use rollo::{
    error::Error,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use std::convert::TryInto;
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
    time::sleep,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_write_packet() {
    setup(6666).await;
    sleep(Duration::from_secs(1)).await;

    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    connect.write(&packet().to_vec()).await.unwrap();

    let size = connect.read_u32().await.unwrap();
    let cmd = connect.read_u16().await.unwrap();
    let payload = connect.read_u32().await.unwrap();

    assert_eq!(size, 4);
    assert_eq!(cmd, 6);
    assert_eq!(payload, 2021);
}

fn packet() -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.put_u32(4);
    bytes.put_u16(6);
    bytes.put_u32(2021);

    bytes
}

async fn setup(port: u32) -> JoinHandle<()> {
    let world = Box::new(MyWorld {});
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

    async fn on_message(world_session: &Arc<Self>, _world: &'static MyWorld, packet: Packet) {
        let packet = packet.freeze();
        assert_eq!(
            u32::from_be_bytes(packet.payload.clone().unwrap()[0..4].try_into().unwrap()),
            2021
        );
        assert_eq!(6, packet.cmd);

        world_session
            .socket_tools
            .send(packet.cmd, Some(packet.payload.as_ref().unwrap()));
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
}
