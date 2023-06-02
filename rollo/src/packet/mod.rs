use bytes::BufMut;
use easy_pool::{PoolObjectContainer, PoolSegQueue};
use once_cell::sync::Lazy;
use std::{mem, sync::Arc};

/// Represents a message with a command and a payload.
#[derive(Debug)]
pub struct Packet {
    /// The command of the message.
    pub cmd: u16,
    /// The payload of the message.
    pub payload: Option<PoolObjectContainer<Vec<u8>>>,
}

impl Packet {
    /// Creates a new Packet with the given command and payload.
    pub(crate) const fn new(cmd: u16, payload: Option<PoolObjectContainer<Vec<u8>>>) -> Self {
        Self { cmd, payload }
    }

    /// Converts the Packet to an Arc<Packet>.
    ///
    /// # Examples
    ///
    /// ```
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

/// Converts a command and a payload to a byte buffer.
///
/// # Examples
///
/// ```
/// use rollo::packet::to_bytes;
///
/// // Cmd 10 with a payload
/// let result = to_bytes(10, Some(&[1, 1, 1]));
/// // You can now send it to the player.
/// ```
pub fn to_bytes(cmd: u16, payload: Option<&[u8]>) -> PoolObjectContainer<Vec<u8>> {
    let payload_size = payload.as_ref().map_or_else(|| 0, |p| p.as_ref().len());
    let size = payload_size as u32;
    let target_capacity = HEADER_SIZE + size as usize;
    let mut vec = POOL_VEC.create();
    debug_assert!(vec.is_empty());

    vec.reserve_exact(target_capacity);
    debug_assert!(vec.capacity() >= target_capacity);

    vec.put_u32(size);
    vec.put_u16(cmd);

    if let Some(payload) = payload {
        vec.extend(payload.as_ref());
    }

    debug_assert!(vec.len() == target_capacity);

    vec
}

static POOL_VEC: Lazy<Arc<PoolSegQueue<Vec<u8>>>> = Lazy::new(|| Arc::new(PoolSegQueue::new(4096)));

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

    #[test]
    fn test_to_bytes_loop() {
        for i in 0..25 {
            if i % 2 == 0 {
                let mut x = POOL_VEC.create();
                x.extend_from_slice(&[0, 5, 5, 25]);
                drop(x);
            }

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
}
