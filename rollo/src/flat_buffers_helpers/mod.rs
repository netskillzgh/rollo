use std::sync::atomic::Ordering;

use crate::server::world_socket_mgr::ACTIVE_SOCKETS;
use crossbeam_queue::SegQueue;
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use lazy_static::lazy_static;

lazy_static! {
    static ref BUILDERS: SegQueue<CustomFlatBufferBuilder> = SegQueue::new();
}

pub fn get_builder() -> CustomFlatBufferBuilder {
    if let Some(builder) = BUILDERS.pop() {
        builder
    } else {
        CustomFlatBufferBuilder::new()
    }
}

pub struct CustomFlatBufferBuilder {
    pub builder: FlatBufferBuilder<'static>,
}

impl CustomFlatBufferBuilder {
    fn new() -> Self {
        CustomFlatBufferBuilder {
            builder: FlatBufferBuilder::new(),
        }
    }

    fn from_builder(builder: FlatBufferBuilder<'static>) -> Self {
        Self { builder }
    }

    pub fn finish<T>(&mut self, root: WIPOffset<T>) -> &[u8] {
        self.builder.finish(root, None);

        self.builder.finished_data()
    }
}

impl Drop for CustomFlatBufferBuilder {
    fn drop(&mut self) {
        let limit: usize = (ACTIVE_SOCKETS.load(Ordering::Relaxed) as usize + 20) * 2;

        if BUILDERS.len() < limit {
            let mut builder = std::mem::take(&mut self.builder);
            builder.reset();
            BUILDERS.push(Self::from_builder(builder));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop() {
        assert_eq!(ACTIVE_SOCKETS.load(Ordering::Relaxed), 20);
        assert_eq!(BUILDERS.len(), 0);

        {
            CustomFlatBufferBuilder::new();
            CustomFlatBufferBuilder::new();
        }

        assert_eq!(BUILDERS.len(), 2);

        {
            CustomFlatBufferBuilder::new();
            CustomFlatBufferBuilder::new();
        }

        assert_eq!(BUILDERS.len(), 4);
    }

    /* #[test]
    fn test_get_builder() {
        (0..BUILDERS.len()).into_iter().for_each(|_| {
            BUILDERS.pop();
        });
        assert!(BUILDERS.is_empty());
        BUILDERS.push(CustomFlatBufferBuilder::new());
        BUILDERS.push(CustomFlatBufferBuilder::new());

        assert_eq!(BUILDERS.len(), 2);

        BUILDERS.pop();

        assert_eq!(BUILDERS.len(), 1);
    } */
}
