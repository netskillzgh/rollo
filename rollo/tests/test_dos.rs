#![cfg(feature = "full")]
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use rollo::{
    error::Error,
    packet::Packet,
    server::{DosPolicy, ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
};
use rollo_macros::world_time;
use std::{
    convert::TryInto,
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedSender},
    task::JoinHandle,
    time::{sleep, timeout},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_dos_attack() {
    let (sender, mut rx) = unbounded_channel();
    setup(6666, sender).await;
    sleep(Duration::from_secs(1)).await;

    // Packet
    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    for i in 0..8 {
        let _ = connect.write(&packet(i, 5).to_vec()).await;
    }

    assert_eq!(
        timeout(Duration::from_secs(20), rx.recv())
            .await
            .unwrap()
            .unwrap(),
        5
    );

    assert!(connect.read_u16().await.is_err());

    // Global
    let mut connect = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    connect.set_nodelay(true).unwrap();

    for i in 0..16 {
        let _ = connect.write(&packet(i, 6).to_vec()).await;
    }

    assert_eq!(
        timeout(Duration::from_secs(20), rx.recv())
            .await
            .unwrap()
            .unwrap(),
        6
    );

    assert!(connect.read_u16().await.is_err());
}

fn packet(number: u16, cmd: u16) -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.put_u32(2);
    bytes.put_u16(cmd);
    bytes.put_u16(number);

    bytes
}

async fn setup(port: u32, sender: UnboundedSender<u16>) -> JoinHandle<()> {
    let world = Box::new(MyWorld {
        sender,
        time: AtomicI64::new(0),
    });
    let world = Box::leak(world);

    let mut server = WorldSocketMgr::new(world);

    tokio::spawn(async move {
        server
            .start_game_loop(Duration::from_millis(50))
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

    async fn on_dos_attack(world_session: &Arc<Self>, world: &'static MyWorld, cmd: u16) {
        world.sender.send(cmd).unwrap();
        world_session.socket_tools().close().unwrap();
    }

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &'static MyWorld, packet: Packet) {
        let n = packet.payload.unwrap();
        if packet.cmd == 5 {
            assert!(u16::from_be_bytes(n[0..2].try_into().unwrap()) <= 5)
        } else {
            assert!(u16::from_be_bytes(n[0..2].try_into().unwrap()) <= 10)
        }
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

#[world_time]
struct MyWorld {
    sender: UnboundedSender<u16>,
}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;

    fn global_limit(&self) -> (u16, u32) {
        (10, 5000)
    }

    fn get_packet_limit(&self, cmd: u16) -> (u16, u32, rollo::server::DosPolicy) {
        if cmd == 5 {
            return (5, 500, DosPolicy::Close);
        }

        (1000, 5000, DosPolicy::Close)
    }
}
