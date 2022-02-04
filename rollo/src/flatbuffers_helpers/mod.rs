use easy_pool::Clear;
use easy_pool::PoolArrayQueue;
pub use flatbuffers;
pub use flatbuffers::{FlatBufferBuilder, WIPOffset};
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static FLAT_BUFFER_BUILDER_GENERATOR: Lazy<Arc<PoolArrayQueue<CustomFlatBuffersBuilder>>> =
    Lazy::new(|| Arc::new(PoolArrayQueue::new(1024)));

pub struct CustomFlatBuffersBuilder {
    pub builder: FlatBufferBuilder<'static>,
}

impl CustomFlatBuffersBuilder {
    fn new() -> Self {
        CustomFlatBuffersBuilder {
            builder: FlatBufferBuilder::new(),
        }
    }

    #[allow(dead_code)]
    pub fn finish<T>(&mut self, root: WIPOffset<T>) -> &[u8] {
        self.builder.finish(root, None);

        self.builder.finished_data()
    }
}

impl Default for CustomFlatBuffersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Clear for CustomFlatBuffersBuilder {
    fn clear(&mut self) {
        self.builder.reset();
    }
}
