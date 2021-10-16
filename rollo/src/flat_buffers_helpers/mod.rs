use std::sync::atomic::Ordering;

use crate::server::world_socket_mgr::ACTIVE_SOCKETS;
use crossbeam_queue::SegQueue;
use flatbuffers::{FlatBufferBuilder, WIPOffset};

static BUILDERS: SegQueue<CustomFlatBufferBuilder> = SegQueue::new();

pub fn generate_builders(number: u32) {
    (0..=number).into_iter().for_each(|_| {
        BUILDERS.push(CustomFlatBufferBuilder::new());
    });
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

    const fn from_builder(builder: FlatBufferBuilder<'static>) -> Self {
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
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn test_drop() {
        let before = BUILDERS.len();

        {
            CustomFlatBufferBuilder::new();
            CustomFlatBufferBuilder::new();
        }

        assert_eq!(BUILDERS.len(), before + 2);

        {
            CustomFlatBufferBuilder::new();
            CustomFlatBufferBuilder::new();
        }

        assert_eq!(BUILDERS.len(), before + 4);
    }

    #[test]
    #[serial]
    fn test_get_builder() {
        BUILDERS.push(CustomFlatBufferBuilder::new());
        let before = BUILDERS.len();
        assert!(before > 0);
        let first = get_builder();
        assert_eq!(before - 1, BUILDERS.len());
        let mut list: Vec<_> = (0..BUILDERS.len())
            .into_iter()
            .map(|_| get_builder())
            .collect();
        assert!(BUILDERS.is_empty());
        let second = get_builder();
        drop(first);
        drop(second);
        list.clear();
        assert_eq!(before + 1, BUILDERS.len());
    }
}
