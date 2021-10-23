use super::{
    tls::load_config,
    world::World,
    world_session::{SocketTools, WorldSession},
    world_socket::WorldSocket,
};
use crate::game::game_loop::GameLoop;
use crate::{
    error::{Error, Result},
    game::GameTime,
};
use crossbeam::atomic::AtomicCell;
use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
    task,
    time::timeout,
};
use tokio_rustls::TlsAcceptor;

/// World Socket Manager
#[derive(Debug, Clone)]
pub struct WorldSocketMgr<W>
where
    W: Send + Sync + 'static + World,
{
    world: &'static W,
    counter: u64,
    configuration: WorldSocketConfiguration,
    game_time: &'static AtomicCell<GameTime>,
}

impl<W> WorldSocketMgr<W>
where
    W: Send + Sync + 'static + World,
{
    /// Create WorldSocketMgr with default configuration
    pub fn new(world: &'static W) -> Self {
        Self {
            world,
            counter: 0,
            configuration: WorldSocketConfiguration::default(),
            game_time: Box::leak(Box::new(AtomicCell::new(GameTime::new()))),
        }
    }

    /// Create WorldSocketMgr with custom configuration
    pub fn with_configuration(
        world: &'static W,
        configuration: WorldSocketConfiguration,
        game_time: &'static AtomicCell<GameTime>,
    ) -> Self {
        Self {
            world,
            counter: 0,
            configuration,
            game_time,
        }
    }

    /// Start the GameLoop with an interval.
    pub fn start_game_loop(&mut self, interval: Duration) -> &mut Self {
        let world = self.world;
        let game_time = self.game_time;
        tokio::spawn(async move {
            let mut game_loop = GameLoop::new(interval);
            game_loop.start(world, Some(game_time)).await;
        });

        self
    }

    /// Start TCP Server
    pub async fn start_network<'a>(
        &mut self,
        addr: impl AsRef<str>,
        security: ListenerSecurity<'_>,
    ) -> Result<()> {
        let address = addr.as_ref().parse::<String>().unwrap();
        let listener = TcpListener::bind(&address).await.unwrap();

        let mut tls_acceptor: Option<TlsAcceptor> = None;

        if let ListenerSecurity::Tls(certificate, key) = security {
            let config = load_config(certificate, key).unwrap();
            tls_acceptor = Some(TlsAcceptor::from(Arc::new(config)));
        }

        let no_delay = self.configuration.no_delay;

        W::on_start(self.game_time).await;

        loop {
            if let Ok((mut socket, addr)) = listener.accept().await {
                self.counter += 1;
                let id = self.counter;
                let acceptor = tls_acceptor.clone();

                let world = self.world;
                let game_time = self.game_time;
                tokio::spawn(async move {
                    if Self::set_up_socket(&mut socket, no_delay).is_ok() {
                        if let Ok((reader, writer)) = Self::try_tls(acceptor, socket).await {
                            Self::create_socket(addr, world, id, reader, writer, game_time).await;
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
        game_time: &'static AtomicCell<GameTime>,
    ) where
        S: AsyncRead + AsyncWrite,
    {
        let (tx, rx) = unbounded_channel();
        let socket_tools = SocketTools::new(socket_addr, tx, id);

        if let Ok(world_session) = W::WorldSessionimplementer::on_open(socket_tools, world).await {
            let mut world_socket = WorldSocket::new(Arc::clone(&world_session), world);
            world_socket.handle(rx, reader, writer, game_time).await;
            W::WorldSessionimplementer::on_close(&world_session, world).await;
        }
    }

    fn set_up_socket(socket: &mut TcpStream, no_delay: bool) -> Result<()> {
        if no_delay {
            socket.set_nodelay(true).map_err(|_| Error::NoDelayError)
        } else {
            Ok(())
        }
    }

    const TIMEOUT_TLS: u64 = 15;

    async fn try_tls<S>(
        tls_acceptor: Option<TlsAcceptor>,
        socket: S,
    ) -> Result<(BufReader<ReadHalf<S>>, WriteHalf<S>)>
    where
        S: AsyncWrite + AsyncRead + Unpin,
    {
        if let Some(acceptor) = tls_acceptor.clone() {
            let socket = timeout(
                Duration::from_secs(Self::TIMEOUT_TLS),
                acceptor.accept(socket),
            )
            .await
            .map_err(|_| Error::TlsAcceptTimeout)?
            .map_err(|_| Error::TlsAccept)?;
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

/// Tcp or Tcp/Tls
#[derive(Debug)]
pub enum ListenerSecurity<'a> {
    Tcp,
    Tls(&'a Path, &'a Path),
}

/// Socket Configuration
#[derive(Debug, Clone, Copy)]
pub struct WorldSocketConfiguration {
    no_delay: bool,
}

impl WorldSocketConfiguration {
    ///```rust, no_run
    /// use rollo::server::WorldSocketConfiguration;
    /// let conf = WorldSocketConfiguration::new(true);
    /// ```
    pub fn new(no_delay: bool) -> Self {
        Self { no_delay }
    }
}

impl Default for WorldSocketConfiguration {
    fn default() -> Self {
        Self { no_delay: true }
    }
}
