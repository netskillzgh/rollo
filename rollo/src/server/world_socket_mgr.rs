use crate::error::Error;

cfg_game! {
    use crate::game::game_loop::GameLoop;
}

use super::{
    tls::load_config,
    world::WorldI,
    world_session::{SocketTools, WorldSessionI},
    world_socket::WorldSocket,
};
use lazy_static::lazy_static;
use std::{
    net::SocketAddr,
    path::Path,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
    task,
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tracing::info;

lazy_static! {
    pub(crate) static ref ACTIVE_SOCKETS: AtomicU64 = AtomicU64::new(0);
}

#[derive(Debug, Clone)]
pub struct WorldSocketMgr<W>
where
    W: Send + Sync + 'static + WorldI,
{
    world: &'static W,
    counter: u64,
    configuration: WorldSocketConfiguration,
}

impl<W> WorldSocketMgr<W>
where
    W: Send + Sync + 'static + WorldI,
{
    /// Create WorldSocketMgr with default configuration
    pub fn new(world: &'static W) -> Self {
        Self {
            world,
            counter: 0,
            configuration: WorldSocketConfiguration::default(),
        }
    }

    /// Create WorldSocketMgr with custom configuration
    pub fn with_configuration(world: &'static W, configuration: WorldSocketConfiguration) -> Self {
        Self {
            world,
            counter: 0,
            configuration,
        }
    }

    cfg_game! {
        /// Start the GameLoop with an interval (tick rate).
        pub fn start_game_loop(&mut self, interval: i64) -> &mut Self {
            let world = self.world;
            tokio::spawn(async move {
                let mut game_loop = GameLoop::new(interval);
                game_loop.start(world).await;
            });

            self
        }
    }

    /// Start TCP Server
    pub async fn start_network(
        &mut self,
        addr: impl AsRef<str>,
        security: ListenerSecurity<'_>,
    ) -> Result<(), Error> {
        let address = addr.as_ref().parse::<String>().unwrap();
        let listener = TcpListener::bind(&address).await.unwrap();

        let mut tls_acceptor: Option<TlsAcceptor> = None;

        if let ListenerSecurity::Tls(certificate, key) = security {
            let config = load_config(certificate, key).unwrap();
            tls_acceptor = Some(TlsAcceptor::from(Arc::new(config)));
        }

        info!("Server started!");

        loop {
            if let Ok((mut socket, addr)) = listener.accept().await {
                self.counter += 1;
                let id = self.counter;
                let acceptor = tls_acceptor.clone();

                let world = self.world;
                tokio::spawn(async move {
                    if Self::set_up_socket(&mut socket).is_ok() {
                        if let Ok((reader, writer)) = Self::try_tls(acceptor, socket).await {
                            Self::create_socket(addr, world, id, reader, writer).await;
                        }
                    }
                });
            }

            task::yield_now().await;
        }
    }

    async fn create_socket<S>(
        socket_addr: SocketAddr,
        world: &'static W,
        id: u64,
        reader: BufReader<ReadHalf<S>>,
        writer: WriteHalf<S>,
    ) where
        S: AsyncRead + AsyncWrite,
    {
        info!("New connection");
        let (tx, rx) = unbounded_channel();
        let socket_tools = SocketTools::new(socket_addr, tx, id);

        if let Ok(world_session) = W::WorldSessionimplementer::on_open(socket_tools, world).await {
            let mut world_socket = WorldSocket::new(Arc::clone(&world_session), world);
            world_socket.handle(rx, reader, writer).await;
            W::WorldSessionimplementer::on_close(&world_session, world).await;
        }
    }

    fn set_up_socket(socket: &mut TcpStream) -> Result<(), Error> {
        socket.set_nodelay(true).map_err(|_| Error::DosProtection)
    }

    const TIMEOUT_TLS: u64 = 15;

    async fn try_tls<S>(
        tls_acceptor: Option<TlsAcceptor>,
        socket: S,
    ) -> Result<(BufReader<ReadHalf<S>>, WriteHalf<S>), Error>
    where
        S: AsyncWrite + AsyncRead + Unpin,
    {
        if let Some(acceptor) = tls_acceptor.clone() {
            let socket = timeout(
                Duration::from_secs(Self::TIMEOUT_TLS),
                acceptor.accept(socket),
            )
            .await
            .map_err(|_| Error::DosProtection)?
            .map_err(|_| Error::TlsAcceptTimeout)?;
            Ok(Self::split_socket(socket.into_inner().0))
        } else {
            Ok(Self::split_socket(socket))
        }
    }

    fn split_socket<S>(socket: S) -> (BufReader<ReadHalf<S>>, WriteHalf<S>)
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let (reader, writer) = tokio::io::split(socket);
        let reader = BufReader::new(reader);

        (reader, writer)
    }
}

#[derive(Debug)]
pub enum ListenerSecurity<'a> {
    Tcp,
    Tls(&'a Path, &'a Path),
}

#[derive(Debug, Clone)]
pub struct WorldSocketConfiguration {
    no_delay: bool,
}

impl WorldSocketConfiguration {
    pub fn new(no_delay: bool) -> Self {
        Self { no_delay }
    }
}

impl Default for WorldSocketConfiguration {
    fn default() -> Self {
        Self { no_delay: true }
    }
}
