use async_trait::async_trait;
use bytes::Bytes;
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
use tracing::error;

use crate::{
    error::Error,
    packet::{to_bytes, Packet},
};

use super::world_socket::WriterMessage;

#[async_trait]
pub trait WorldSession<T: Send + Sync>
where
    Self: Sync + Send,
{
    /// On Connection Open
    async fn on_open(socket_tools: SocketTools, world: &'static T) -> Result<Arc<Self>, Error>;
    fn socket_tools(&self) -> &SocketTools;
    async fn on_message(world_session: &Arc<Self>, world: &'static T, packet: Packet);
    /// On Connection Close
    async fn on_close(world_session: &Arc<Self>, world: &'static T);
    async fn on_dos_trigger(_world_session: &Arc<Self>, _world: &'static T, _cmd: u16) {}
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
    pub fn send(&self, cmd: u16, payload: Option<&[u8]>) {
        if !self.is_closed() {
            if let Ok(bytes) = to_bytes(cmd, payload) {
                if let Err(error) = self.tx.send(WriterMessage::Bytes(bytes.freeze())) {
                    error!("Error when sending the packet. {}", error);
                }
            } else {
                error!("Error when transforming the packet");
            }
        }
    }

    /// Send Bytes(Packet) to the session
    pub fn send_data(&self, bytes: Bytes) {
        if !self.is_closed() && self.tx.send(WriterMessage::Bytes(bytes)).is_err() {
            error!("Error when sending packet");
        }
    }

    pub fn get_latency(&self) -> i64 {
        self.latency.load(Ordering::Acquire)
    }

    /// Close the session
    pub fn close(&self) -> Result<(), Error> {
        self.tx
            .send(WriterMessage::Close)
            .map_err(|_| Error::Channel)
    }

    /// Close the session with a delay
    pub fn close_with_delay(&self, delay: Duration) -> Result<(), Error> {
        self.tx
            .send(WriterMessage::CloseDelayed(delay))
            .map_err(|_| Error::Channel)
    }

    /// Is connection close ?
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

impl Clone for SocketTools {
    fn clone(&self) -> Self {
        Self {
            latency: AtomicI64::new(self.latency.load(Ordering::Relaxed)),
            ..self.clone()
        }
    }
}
