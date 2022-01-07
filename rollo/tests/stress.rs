#![cfg(feature = "full")]
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use rollo::{
    error::Error,
    packet::Packet,
    server::{DosPolicy, ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
    time::sleep,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_stress() {
    setup(6666).await;
    sleep(Duration::from_secs(1)).await;

    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    for i in 0..500 {
        connect.write(&packet(i).to_vec()).await.unwrap();
    }

    let mut counter = 0;
    for i in 0..500 {
        let size = connect.read_u32().await.unwrap();
        let cmd = connect.read_u16().await.unwrap();
        let payload = connect.read_u32().await.unwrap();

        assert_eq!(size, 4);
        assert_eq!(cmd, 6);
        assert_eq!(payload, i);
        counter += 1;
    }

    assert_eq!(counter, 500);
}

fn packet(i: u32) -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.put_u32(4);
    bytes.put_u16(6);
    bytes.put_u32(i);

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
        assert!(packet.payload.is_some());
        assert_eq!(6, packet.cmd);
        world_session
            .socket_tools
            .send(packet.cmd, Some(&packet.payload.unwrap()));
        world_session.socket_tools.flush();
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;

    fn global_limit(&self) -> (u16, u32) {
        (10000, 15000)
    }

    fn get_packet_limit(&self, _cmd: u16) -> (u16, u32, rollo::server::DosPolicy) {
        (1000, 12000, DosPolicy::None)
    }
}
