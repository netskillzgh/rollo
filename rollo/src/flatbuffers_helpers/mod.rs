use easy_pool::Clear;
use easy_pool::PoolArrayQueue;
pub use flatbuffers;
pub use flatbuffers::{FlatBufferBuilder, WIPOffset};
use once_cell::sync::Lazy;
use std::sync::Arc;

/// A thread-safe global instance of a pool of `CustomFlatBuffersBuilder`s.
pub static FLAT_BUFFER_BUILDER_GENERATOR: Lazy<Arc<PoolArrayQueue<CustomFlatBuffersBuilder>>> =
    Lazy::new(|| Arc::new(PoolArrayQueue::new(1024)));

/// A custom `FlatBufferBuilder` struct.
pub struct CustomFlatBuffersBuilder {
    pub builder: FlatBufferBuilder<'static>,
}

impl CustomFlatBuffersBuilder {
    /// Creates a new instance of the `CustomFlatBuffersBuilder` struct.
    fn new() -> Self {
        CustomFlatBuffersBuilder {
            builder: FlatBufferBuilder::new(),
        }
    }

    /// Finishes building a FlatBuffer and returns the resulting byte array.
    pub fn finish<T>(&mut self, root: WIPOffset<T>) -> &[u8] {
        self.builder.finish(root, None);

        self.builder.finished_data()
    }
}

impl Default for CustomFlatBuffersBuilder {
    /// Implements the `Default` trait to create a new instance of the `CustomFlatBuffersBuilder` struct.
    fn default() -> Self {
        Self::new()
    }
}

impl Clear for CustomFlatBuffersBuilder {
    /// Implements the `Clear` trait to reset the builder's state.
    fn clear(&mut self) {
        self.builder.reset();
    }
}