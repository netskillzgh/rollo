use std::convert::TryFrom;
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result};

pub(crate) const MAX_SIZE: usize = 1024 * 14;

pub(crate) struct Reader<'a, R>
where
    R: AsyncReadExt + Unpin,
{
    size: [u8; 4],
    cmd: [u8; 2],
    buffer: &'a mut R,
    payload: [u8; MAX_SIZE],
}

impl<'a, R> Reader<'a, R>
where
    R: AsyncReadExt + Unpin,
{
    pub(crate) fn new(buffer: &'a mut R) -> Self {
        Self {
            size: [0; 4],
            cmd: [0; 2],
            buffer,
            payload: [0; MAX_SIZE],
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

    pub(crate) async fn read_payload(&mut self, size: usize) -> Result<Option<Vec<u8>>>
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

        let payload = Vec::from(&self.payload[0..size]);

        Ok(Some(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_parse_size() {
        let mut buffer = Cursor::new(vec![0x00, 0x00, 0x01, 0x0b]);
        let mut reader = Reader::new(&mut buffer);
        let size = reader.read_size().await.unwrap();
        assert_eq!(size, 267);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[should_panic]
    async fn test_parse_size_fail_zero() {
        let mut buffer = Cursor::new(vec![]);
        let mut reader = Reader::new(&mut buffer);
        reader.read_size().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_parse_op_code() {
        let mut buffer = Cursor::new(vec![0x00, 0xc1]);
        let mut reader = Reader::new(&mut buffer);
        let op_code = reader.read_cmd().await.unwrap();
        assert_eq!(op_code, 193);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[should_panic]
    async fn test_fail_parse_size_negative_number() {
        let mut buffer = Cursor::new(vec![0xff, 0x7c]);
        let mut reader = Reader::new(&mut buffer);
        reader.read_size().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_parse_content() {
        let content = vec![0x00, 0xc1];
        let mut buffer = Cursor::new(content.clone());
        let mut reader = Reader::new(&mut buffer);
        let result = reader.read_payload(2).await.unwrap();
        assert_eq!(result.unwrap(), content);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_parse_content_fail() {
        let content = vec![];
        let mut buffer = Cursor::new(content.clone());
        let mut reader = Reader::new(&mut buffer);
        let result = reader.read_payload(0).await.unwrap();
        assert_eq!(result, None);
    }
}
