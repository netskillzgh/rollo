use super::world_socket::WriterMessage;
use crate::error::{Error, Result};
use crate::packet::{to_bytes, Packet};
use async_trait::async_trait;
use bytes::Bytes;
#[cfg(feature = "flatbuffers_helpers")]
use flatbuffers::FlatBufferBuilder;
use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;

/// Events for a WorldSession
#[async_trait]
pub trait WorldSession<T: Send + Sync>
where
    Self: Sync + Send,
{
    /// On Connection Open
    async fn on_open(socket_tools: SocketTools, world: &'static T) -> Result<Arc<Self>>;
    fn socket_tools(&self) -> &SocketTools;
    async fn on_message(world_session: &Arc<Self>, world: &'static T, packet: Packet);
    /// On Connection Close
    async fn on_close(world_session: &Arc<Self>, world: &'static T);
    async fn on_dos_attack(_world_session: &Arc<Self>, _world: &'static T, _cmd: u16) {}
}

/// Send packets, latency, SocketTools etc.
#[derive(Debug)]
pub struct SocketTools {
    pub socket_addr: SocketAddr,
    pub(crate) tx: UnboundedSender<WriterMessage>,
    pub id: u64,
    pub(crate) latency: AtomicI64,
}

impl SocketTools {
    pub(crate) fn new(
        socket_addr: SocketAddr,
        tx: UnboundedSender<WriterMessage>,
        id: u64,
    ) -> Self {
        Self {
            socket_addr,
            tx,
            id,
            latency: AtomicI64::new(0),
        }
    }

    /// Send a packet to the session
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     socket.send(1, None);
    /// }
    /// ```
    pub fn send(&self, cmd: u16, payload: Option<&[u8]>) {
        if !self.is_closed() {
            let bytes = to_bytes(cmd, payload);
            if self.tx.send(WriterMessage::Bytes(bytes.freeze())).is_err() {
                log::error!("Can't send the data to the channel.");
            }
        }
    }

    /// Send Bytes(Packet) to the session
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    /// use rollo::packet::to_bytes;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     let bytes = to_bytes(1, None).freeze();
    ///     socket.send_data(bytes);
    /// }
    /// ```
    pub fn send_data(&self, bytes: Bytes) {
        if !self.is_closed() && self.tx.send(WriterMessage::Bytes(bytes)).is_err() {
            log::error!("Can't send the data to the channel.");
        }
    }

    /// get Latency
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     let latency = socket.get_latency();
    /// }
    /// ```
    pub fn get_latency(&self) -> i64 {
        self.latency.load(Ordering::Acquire)
    }

    #[cfg(feature = "flatbuffers_helpers")]
    pub fn send_flatbuffers<
        F: 'static + Fn(&mut FlatBufferBuilder<'static>) -> Bytes + Send + Sync,
    >(
        &self,
        f: F,
    ) {
        if self
            .tx
            .send(WriterMessage::SendFlatbuffers(Box::new(f)))
            .is_err()
        {
            log::error!("Can't send the data to the channel.");
        }
    }

    /// Close the session
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     if socket.close().is_err() {
    ///         println!("Error when closing the session");
    ///     }
    /// }
    /// ```
    pub fn close(&self) -> Result<()> {
        self.tx
            .send(WriterMessage::Close)
            .map_err(|_| Error::Channel)
    }

    /// Close the session with a delay
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    /// use std::time::Duration;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     if socket.close_with_delay(Duration::from_secs(1)).is_err() {
    ///         println!("Error when closing the session");
    ///     }
    /// }
    /// ```
    pub fn close_with_delay(&self, delay: Duration) -> Result<()> {
        self.tx
            .send(WriterMessage::CloseDelayed(delay))
            .map_err(|_| Error::Channel)
    }

    /// Is connection close ?
    /// ## Examples
    /// ```rust, no_run
    /// use rollo::server::SocketTools;
    ///
    /// fn on_message(socket: SocketTools) {
    ///     if socket.is_closed() {
    ///     // Do something
    ///     }
    /// }
    /// ```
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

impl Clone for SocketTools {
    fn clone(&self) -> Self {
        Self {
            latency: AtomicI64::new(self.latency.load(Ordering::Relaxed)),
            tx: self.tx.clone(),
            id: self.id,
            socket_addr: self.socket_addr,
        }
    }
}

impl PartialEq for SocketTools {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for SocketTools {}
