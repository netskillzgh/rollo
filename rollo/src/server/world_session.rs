use super::world_socket::WriterMessage;
use crate::error::{Error, Result};
use crate::packet::{to_bytes, Packet};
use crate::server::world_socket::ContainerBytes;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use easy_pool::PoolObjectContainer;
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

/// A trait for defining events for a WorldSession.
#[async_trait]
pub trait WorldSession<T: Send + Sync>
where
    Self: Sync + Send,
{
    /// Called when the connection is opened.
    async fn on_open(socket_tools: SocketTools, world: &'static T) -> Result<Arc<Self>>;

    /// Returns a reference to the SocketTools object.
    fn socket_tools(&self) -> &SocketTools;

    /// Called when a message is received.
    async fn on_message(world_session: &Arc<Self>, world: &'static T, packet: Packet);

    /// Called when the connection is closed.
    async fn on_close(world_session: &Arc<Self>, world: &'static T);

    /// Called when a Denial of Service (DoS) attack is detected.
    async fn on_dos_attack(_world_session: &Arc<Self>, _world: &'static T, _cmd: u16) {}
}

/// A struct for sending packets, measuring latency, and managing the SocketTools object.
#[derive(Debug)]
pub struct SocketTools {
    /// The socket address.
    pub socket_addr: SocketAddr,

    /// The sender for the writer message.
    pub(crate) tx: UnboundedSender<WriterMessage>,

    /// The ID of the SocketTools object.
    pub id: u64,

    /// The latency of the connection.
    pub(crate) latency: AtomicI64,

    /// Indicates whether the connection is closed.
    closed: AtomicCell<bool>,
}

impl SocketTools {
    /// Creates a new SocketTools object.
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
            closed: AtomicCell::new(false),
        }
    }

    /// Sends a packet to the session.
    pub fn send(&self, cmd: u16, payload: Option<&[u8]>) {
        if !self.is_closed() {
            let bytes = to_bytes(cmd, payload);
            if self
                .tx
                .send(WriterMessage::Send(bytes.into(), true))
                .is_err()
            {
                log::error!("Can't send the data to the channel.");
            }
        }
    }

    /// Sends bytes (Packet) to the session.
    pub fn send_data(&self, bytes: ContainerBytes) {
        if !self.is_closed() && self.tx.send(WriterMessage::Send(bytes, true)).is_err() {
            log::error!("Can't send the data to the channel.");
        }
    }

    /// Writes bytes (Packet) to the session.
    pub fn write_data(&self, bytes: ContainerBytes) {
        if !self.is_closed() && self.tx.send(WriterMessage::Send(bytes, false)).is_err() {
            log::error!("Can't send the data to the channel.");
        }
    }

    /// Flushes the session.
    pub fn flush(&self) {
        if !self.is_closed() && self.tx.send(WriterMessage::Flush).is_err() {
            log::error!("Can't send the data to the channel.");
        }
    }

    /// Returns the latency of the connection.
    pub fn get_latency(&self) -> i64 {
        self.latency.load(Ordering::Acquire)
    }

    /// Closes the session.
    pub fn close(&self) -> Result<()> {
        self.closed.store(true);
        self.tx
            .send(WriterMessage::Close)
            .map_err(|_| Error::Channel)
    }

    /// Closes the session with a delay.
    pub fn close_with_delay(&self, delay: Duration) -> Result<()> {
        self.tx
            .send(WriterMessage::CloseDelayed(delay))
            .map_err(|_| Error::Channel)
    }

    /// Returns true if the connection is closed.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed() || self.closed.load()
    }
}

impl Clone for SocketTools {
    fn clone(&self) -> Self {
        Self {
            latency: AtomicI64::new(self.latency.load(Ordering::Relaxed)),
            tx: self.tx.clone(),
            id: self.id,
            socket_addr: self.socket_addr,
            closed: AtomicCell::new(self.closed.load()),
        }
    }
}

impl PartialEq for SocketTools {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SocketTools {}

impl From<PoolObjectContainer<Vec<u8>>> for ContainerBytes {
    fn from(b: PoolObjectContainer<Vec<u8>>) -> Self {
        ContainerBytes::Raw(b)
    }
}

impl From<Arc<PoolObjectContainer<Vec<u8>>>> for ContainerBytes {
    fn from(b: Arc<PoolObjectContainer<Vec<u8>>>) -> Self {
        ContainerBytes::Arc(b)
    }
}