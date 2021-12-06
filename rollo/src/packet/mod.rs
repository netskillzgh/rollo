use bytes::{BufMut, BytesMut};
use std::{mem, sync::Arc};

/// Message representation Cmd + Payload
#[derive(Debug, Clone)]
pub struct Packet {
    pub cmd: u16,
    pub payload: Option<Vec<u8>>,
}

impl Packet {
    pub(crate) const fn new(cmd: u16, payload: Option<Vec<u8>>) -> Self {
        Self { cmd, payload }
    }

    /// ## Converts Packet to an Arc<Packet>
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::packet::Packet;
    ///
    /// fn on_message(packet: Packet) {
    ///     let packet = packet.freeze();
    /// }
    /// ```
    pub fn freeze(self) -> Arc<Self> {
        Arc::new(self)
    }
}

const HEADER_SIZE: usize = mem::size_of::<u32>() + mem::size_of::<u16>();

/// ## Cmd + Payload to BytesMut
/// ### Examples
/// ```rust, no_run
/// use rollo::packet::to_bytes;
///
/// // Cmd 10 with a payload
/// let result = to_bytes(10, Some(&[1, 1, 1]));
/// // Converts BytesMut into an immutable Bytes.
/// result.freeze();
/// // You can now send it to the player.
/// ```
pub fn to_bytes(cmd: u16, payload: Option<&[u8]>) -> BytesMut {
    let payload_size = payload.as_ref().map_or_else(|| 0, |p| p.as_ref().len());
    let mut buffer = BytesMut::with_capacity(HEADER_SIZE + payload_size);

    let size = payload_size as u32;
    buffer.put_u32(size);
    buffer.put_u16(cmd);
    if let Some(payload) = payload {
        buffer.extend_from_slice(payload.as_ref());
    }

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    #[test]
    fn test_to_bytes() {
        let content = [1, 1, 2];
        let result = to_bytes(1, Some(&content));
        assert_eq!(result.len(), 9);
        let size = u32::from_be_bytes(result[..4].try_into().unwrap());
        assert_eq!(size, 3);
        let op_code = u16::from_be_bytes(result[4..6].try_into().unwrap());
        assert_eq!(op_code, 1);
        assert_eq!(result[6..], content);
    }
}
