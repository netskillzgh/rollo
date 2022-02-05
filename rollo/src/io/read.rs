use crate::error::{Error, Result};
use easy_pool::{PoolObjectContainer, PoolSegQueue};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, sync::Arc};
use tokio::io::AsyncReadExt;

pub(crate) const MAX_SIZE: usize = 1024 * 14;
static POOL_VEC: Lazy<Arc<PoolSegQueue<Vec<u8>>>> = Lazy::new(|| Arc::new(PoolSegQueue::new(4096)));

pub(crate) struct Reader<'a, R>
where
    R: AsyncReadExt + Unpin,
{
    size: [u8; 4],
    cmd: [u8; 2],
    buffer: &'a mut R,
    payload: PoolObjectContainer<Vec<u8>>,
}

impl<'a, R> Reader<'a, R>
where
    R: AsyncReadExt + Unpin,
{
    pub(crate) fn new(buffer: &'a mut R) -> Self {
        let mut vec = POOL_VEC.create();
        vec.resize(MAX_SIZE, 0);
        debug_assert_eq!(vec.len(), MAX_SIZE);

        Self {
            size: [0; 4],
            cmd: [0; 2],
            buffer,
            payload: vec,
        }
    }

    pub(crate) async fn read_size(&mut self) -> Result<usize> {
        self.buffer
            .read_exact(&mut self.size)
            .await
            .map_err(|_| Error::ReadingPacket)?;

        let size = u32::from_be_bytes(self.size);

        let size = usize::try_from(size).map_err(|_| Error::NumberConversion)?;

        Ok(size)
    }

    pub(crate) async fn read_cmd(&mut self) -> Result<u16> {
        self.buffer
            .read_exact(&mut self.cmd)
            .await
            .map_err(|_| Error::ReadingPacket)?;

        let cmd = u16::from_be_bytes(self.cmd);

        Ok(cmd)
    }

    pub(crate) async fn read_payload(
        &mut self,
        size: usize,
    ) -> Result<Option<PoolObjectContainer<Vec<u8>>>>
    where
        R: AsyncReadExt + Unpin,
    {
        if size == 0 {
            return Ok(None);
        }

        let size = self
            .buffer
            .read_exact(&mut self.payload[0..size])
            .await
            .map_err(|_| Error::PacketSize)?;

        if size == 0 {
            return Err(Error::PacketSize);
        }

        let mut payload = POOL_VEC_PACKET.create();
        debug_assert!(payload.is_empty());
        payload.extend_from_slice(&self.payload[0..size]);
        debug_assert!(payload.len() == size);

        Ok(Some(payload))
    }
}

static POOL_VEC_PACKET: Lazy<Arc<PoolSegQueue<Vec<u8>>>> =
    Lazy::new(|| Arc::new(PoolSegQueue::new(4096)));

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_parse_size() {
        let mut buffer = Cursor::new(vec![0x00, 0x00, 0x01, 0x0b]);
        let mut reader = Reader::new(&mut buffer);
        let size = reader.read_size().await.unwrap();
        assert_eq!(size, 267);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_parse_size_fail_zero() {
        let mut buffer = Cursor::new(vec![]);
        let mut reader = Reader::new(&mut buffer);
        reader.read_size().await.unwrap();
    }

    #[tokio::test]
    async fn test_parse_op_code() {
        let mut buffer = Cursor::new(vec![0x00, 0xc1]);
        let mut reader = Reader::new(&mut buffer);
        let op_code = reader.read_cmd().await.unwrap();
        assert_eq!(op_code, 193);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_fail_parse_size_negative_number() {
        let mut buffer = Cursor::new(vec![0xff, 0x7c]);
        let mut reader = Reader::new(&mut buffer);
        reader.read_size().await.unwrap();
    }

    #[tokio::test]
    async fn test_parse_content() {
        let content = vec![0x00, 0xc1];
        let mut buffer = Cursor::new(content.clone());
        let mut reader = Reader::new(&mut buffer);
        let result = reader.read_payload(2).await.unwrap();
        assert_eq!(*result.unwrap(), content);
    }

    #[tokio::test]
    async fn test_parse_content_loop() {
        for i in 0..25 {
            let mut content = vec![0x00, 0xc1];

            if i % 2 == 0 {
                content.push(i);
            }

            let mut buffer = Cursor::new(content.clone());
            let mut reader = Reader::new(&mut buffer);
            let result = reader.read_payload(content.len()).await.unwrap();
            assert_eq!(*result.unwrap(), content);
        }
    }

    #[tokio::test]
    async fn test_parse_content_fail() {
        let content = vec![];
        let mut buffer = Cursor::new(content.clone());
        let mut reader = Reader::new(&mut buffer);
        let result = reader.read_payload(0).await.unwrap();
        assert!(result.is_none());
    }
}
