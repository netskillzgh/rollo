use crate::error::Error;
use crate::io::read::{Reader, MAX_SIZE};
use crate::packet::Packet;

use super::dos_protection::{DosPolicy, DosProtection};
use super::world::World;
use super::world_session::WorldSession;
use super::world_socket_mgr::ACTIVE_SOCKETS;
use bytes::Bytes;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader, ReadHalf, WriteHalf},
    sync::mpsc::UnboundedReceiver,
};
use tokio::{select, task};

#[derive(Debug)]
pub(crate) struct WorldSocket<T, S, W>
where
    T: WorldSession<W> + 'static + Send + Sync,
    W: 'static + Send + Sync + World,
{
    world_session: Arc<T>,
    world: &'static W,
    dos_protection: DosProtection,
    phantom: PhantomData<S>,
}

impl<T, S, W> WorldSocket<T, S, W>
where
    T: WorldSession<W> + 'static + Send + Sync,
    S: AsyncRead + AsyncWrite,
    W: 'static + Send + Sync + World,
{
    async fn dispatch_channel(
        mut rx: UnboundedReceiver<PacketDispatcher>,
        world_session: Arc<T>,
        world: &'static W,
    ) {
        while let Some(message) = rx.recv().await {
            match message {
                PacketDispatcher::Close => break,
                PacketDispatcher::Packet(packet) => {
                    T::on_message(&world_session, world, packet).await
                }
            }

            task::yield_now().await;
        }
    }

    fn dispatch_client_packets(&self) -> (UnboundedSender<PacketDispatcher>, JoinHandle<()>) {
        let (tx, rx) = unbounded_channel();

        let world = self.world;
        let world_session = Arc::clone(&self.world_session);
        let t = tokio::spawn(async move {
            Self::dispatch_channel(rx, world_session, world).await;
        });

        (tx, t)
    }

    pub(crate) async fn handle(
        &mut self,
        rx: UnboundedReceiver<WriterMessage>,
        mut reader: BufReader<ReadHalf<S>>,
        writer: WriteHalf<S>,
    ) where
        S: AsyncWrite + AsyncRead,
    {
        ACTIVE_SOCKETS.fetch_add(1, Ordering::Relaxed);

        let (mut tx_packet, t) = self.dispatch_client_packets();

        select! {
            _ = self.read(&mut reader, &mut tx_packet) => {}
            _ = Self::write(writer, rx) => {}
        }

        if tx_packet.send(PacketDispatcher::Close).is_err() {
            log::info!("Can't close the channel.");
        }

        if t.await.is_err() {}

        ACTIVE_SOCKETS.fetch_sub(1, Ordering::Relaxed);
    }

    fn handle_ping(&self, packet: Packet) -> Result<(), Error> {
        if let Some(content) = packet.payload {
            self.world_session.socket_tools().send(0, Some(&content));
            let latency = parse_ping(content)?;
            self.world_session
                .socket_tools()
                .latency
                .store(latency, Ordering::Relaxed);
            Ok(())
        } else {
            Err(Error::PacketPayload)
        }
    }

    pub(crate) fn new(world_session: Arc<T>, world: &'static W) -> Self {
        Self {
            world_session,
            dos_protection: DosProtection::new(),
            phantom: PhantomData,
            world,
        }
    }

    async fn process_packet(
        &mut self,
        reader: &mut Reader<'_, BufReader<ReadHalf<S>>>,
        tx: &mut UnboundedSender<PacketDispatcher>,
    ) -> Result<(), Error> {
        let result = {
            if let Ok(result) = timeout(Duration::from_secs(20), self.read_packet(reader)).await {
                match result {
                    Ok(packet) if packet.cmd == 0 => self.handle_ping(packet),
                    Ok(packet) => tx
                        .send(PacketDispatcher::Packet(packet))
                        .map_err(|_| Error::Channel),
                    Err(error) => Err(error),
                }
            } else {
                Err(Error::TimeoutReading)
            }
        };

        result
    }

    async fn read(
        &mut self,
        buffer: &mut BufReader<ReadHalf<S>>,
        tx: &mut UnboundedSender<PacketDispatcher>,
    ) {
        let mut reader = Reader::new(buffer);
        loop {
            let result = self.process_packet(&mut reader, tx).await;

            if result.is_err() {
                break;
            }

            task::yield_now().await;
        }
    }

    async fn read_packet(
        &mut self,
        reader: &mut Reader<'_, BufReader<ReadHalf<S>>>,
    ) -> Result<Packet, Error> {
        let size = reader.read_size().await?;
        let cmd = reader.read_cmd().await?;

        let (limit, size_limit, policy) = self.world.get_packet_limits(cmd);

        if (size as u32) >= size_limit || size >= MAX_SIZE {
            return Err(Error::PacketSize);
        }

        if !self
            .dos_protection
            .evaluate_cmd(cmd, limit, self.world.time())
        {
            WorldSession::on_dos_trigger(&self.world_session, self.world, cmd).await;
            match policy {
                DosPolicy::Close => {
                    if self.world_session.socket_tools().close().is_err() {
                        log::error!("Error when closing the channel.");
                    }
                    return Err(Error::DosProtection);
                }
                DosPolicy::Log => {
                    log::info!("Potential dos attack detected for Cmd {}.", cmd);
                }
                DosPolicy::None => {}
            }
        }

        if size == 0 {
            Ok(Packet::new(cmd, None))
        } else {
            let payload = reader.read_payload(size).await?;

            Ok(Packet::new(cmd, payload))
        }
    }

    async fn write(mut writer: WriteHalf<S>, mut rx: UnboundedReceiver<WriterMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                WriterMessage::Close => break,
                WriterMessage::CloseDelayed(duration) => {
                    sleep(duration).await;
                    break;
                }
                WriterMessage::Bytes(data) => {
                    if data.is_empty() {
                        continue;
                    }

                    if writer.write_all(&data).await.is_err() {
                        break;
                    }

                    task::yield_now().await;
                }
            }
        }
    }
}

fn parse_ping(content: Vec<u8>) -> Result<i64, Error> {
    if content.len() == 16 {
        let middle = content.len() - 8;
        let latency = content[middle..]
            .try_into()
            .map_err(|_| Error::PacketPayload)?;
        let latency = i64::from_be_bytes(latency);
        Ok(latency)
    } else {
        Err(Error::PacketPayload)
    }
}

pub(crate) enum WriterMessage {
    Close,
    CloseDelayed(Duration),
    Bytes(Bytes),
}

pub(crate) enum PacketDispatcher {
    Close,
    Packet(Packet),
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_parse_ping() {
        let mut bytes = BytesMut::new();
        bytes.put_u32(8); // Size
        bytes.put_u16(0); // Cmd
        bytes.put_u16(100); // Ping Date
        bytes.put_i64(75); // Latency

        assert_eq!(parse_ping(bytes.to_vec()).unwrap(), 75);
    }
}
