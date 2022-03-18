use super::dos_protection::{DosPolicy, DosProtection};
use super::world::World;
use super::world_session::WorldSession;
use crate::error::{Error, Result};
use crate::game::GameTime;
use crate::io::read::{Reader, MAX_SIZE};
use crate::packet::Packet;
use crossbeam::atomic::AtomicCell;
use easy_pool::PoolObjectContainer;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::{atomic::Ordering, Arc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task::yield_now;
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
    pub(crate) async fn handle<'a>(
        &'a mut self,
        rx: UnboundedReceiver<WriterMessage>,
        mut reader: BufReader<ReadHalf<S>>,
        writer: WriteHalf<S>,
        game_time: &'static AtomicCell<GameTime>,
        timeout_read: u64,
        world_session: &'a Arc<T>,
    ) where
        S: AsyncWrite + AsyncRead,
    {
        select! {
            _ = self.read(&mut reader, game_time, timeout_read, world_session, self.world) => {}
            _ = Self::write(writer, rx) => {}
        }
    }

    fn handle_ping(&self, packet: Packet) -> Result<()> {
        if let Some(content) = packet.payload {
            self.world_session.socket_tools().send(0, Some(&content));
            let latency = parse_ping(&*content)?;
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

    async fn process_packet<'a>(
        &'a mut self,
        reader: &'a mut Reader<'_, BufReader<ReadHalf<S>>>,
        game_time: &'static AtomicCell<GameTime>,
        timeout_read: u64,
        world_session: &'a Arc<T>,
        world: &'static W,
    ) -> Result<()> {
        let result = {
            if let Ok(result) = timeout(
                Duration::from_secs(timeout_read),
                self.read_packet(reader, game_time),
            )
            .await
            {
                match result {
                    Ok(packet) if packet.cmd == 0 => self.handle_ping(packet),
                    Ok(packet) => {
                        T::on_message(world_session, world, packet).await;
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            } else {
                Err(Error::TimeoutReading)
            }
        };

        result
    }

    async fn read<'a>(
        &'a mut self,
        buffer: &'a mut BufReader<ReadHalf<S>>,
        game_time: &'static AtomicCell<GameTime>,
        timeout_read: u64,
        world_session: &'a Arc<T>,
        world: &'static W,
    ) {
        let mut reader = Reader::new(buffer);
        loop {
            let result = self
                .process_packet(&mut reader, game_time, timeout_read, world_session, world)
                .await;

            if result.is_err() {
                break;
            }

            task::yield_now().await;
        }
    }

    async fn read_packet<'a>(
        &'a mut self,
        reader: &'a mut Reader<'_, BufReader<ReadHalf<S>>>,
        game_time: &'static AtomicCell<GameTime>,
    ) -> Result<Packet> {
        let size = reader.read_size().await?;
        let cmd = reader.read_cmd().await?;

        let (global_amount_limit, global_size_limit) = self.world.global_limit();
        let (packet_amount_limit, packet_size_limit, policy) = self.world.get_packet_limit(cmd);

        if size >= MAX_SIZE || (size as u32) >= packet_size_limit {
            return Err(Error::PacketSize);
        }

        let time = game_time.load().timestamp;

        let global_result = self.dos_protection.evaluate_global_limit(
            time,
            size as u32,
            global_size_limit,
            global_amount_limit,
        );

        if !global_result
            || !self
                .dos_protection
                .evaluate_cmd(cmd, packet_amount_limit, time)
        {
            WorldSession::on_dos_attack(&self.world_session, self.world, cmd).await;

            if !global_result {
                return Err(self.close_dos());
            }

            match policy {
                DosPolicy::Close => {
                    return Err(self.close_dos());
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

    fn close_dos(&self) -> Error {
        if self.world_session.socket_tools().close().is_err() {
            log::error!("Error when closing the channel.");
        }
        Error::DosProtection
    }

    async fn write(writer: WriteHalf<S>, mut rx: UnboundedReceiver<WriterMessage>) {
        let mut writer = BufWriter::new(writer);
        #[cfg(feature = "flatbuffers_helpers")]
        while let Some(message) = rx.recv().await {
            match message {
                WriterMessage::Close => break,
                WriterMessage::CloseDelayed(duration) => {
                    sleep(duration).await;
                    break;
                }
                WriterMessage::Send(data, flush) => {
                    if data.is_empty() {
                        yield_now().await;
                        continue;
                    }

                    if writer.write_all(data.bytes()).await.is_err() {
                        break;
                    }

                    if flush {
                        if let Err(error) = writer.flush().await {
                            log::error!("Error when flushing {:?}", error);
                        }

                        println!(
                            "Sent at {}",
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        );
                    }
                }
                WriterMessage::Flush => {
                    if let Err(error) = writer.flush().await {
                        log::error!("Error when flushing {:?}", error);
                    }
                }
            }

            yield_now().await;
        }
    }
}

fn parse_ping(content: &[u8]) -> Result<i64> {
    if content.len() == 16 {
        let middle = content.len() / 2;
        let latency = content[middle..]
            .try_into()
            .map_err(|_| Error::PacketPayload)?;
        let latency = i64::from_be_bytes(latency);
        Ok(latency)
    } else {
        Err(Error::PacketPayload)
    }
}

pub enum ContainerBytes {
    Raw(PoolObjectContainer<Vec<u8>>),
    Arc(Arc<PoolObjectContainer<Vec<u8>>>),
}

impl ContainerBytes {
    pub fn bytes(&self) -> &[u8] {
        match self {
            ContainerBytes::Raw(b) => &*b,
            ContainerBytes::Arc(b) => &*b,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ContainerBytes::Raw(b) => b.is_empty(),
            ContainerBytes::Arc(b) => b.is_empty(),
        }
    }
}

pub(crate) enum WriterMessage {
    Close,
    Flush,
    CloseDelayed(Duration),
    Send(ContainerBytes, bool),
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

        assert_eq!(parse_ping(&bytes.to_vec()).unwrap(), 75);
    }
}
