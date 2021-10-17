pub use crossbeam_queue::ArrayQueue;
pub use flatbuffers;
pub use flatbuffers::{FlatBufferBuilder, WIPOffset};

#[cfg(feature = "flatbuffers_helpers")]
#[cfg_attr(docsrs, doc(cfg(feature = "flatbuffers_helpers")))]
#[macro_export]
macro_rules! pool_flatbuffers {
    ($l:tt, $n:ident, $f:ident) => {
        use $crate::flatbuffers_helpers::ArrayQueue;
        use $crate::flatbuffers_helpers::{FlatBufferBuilder, WIPOffset};
        use $crate::lazy_static::lazy_static;

        lazy_static! {
            static ref $n: ArrayQueue<CustomFlatBuffersBuilder> = ArrayQueue::new($l);
        }

        #[allow(dead_code)]
        pub fn generate_builders(number: u32) {
            (0..=number).into_iter().for_each(|_| {
                let _ = $n.push(CustomFlatBuffersBuilder::new());
            });
        }

        #[allow(dead_code)]
        pub fn $f() -> CustomFlatBuffersBuilder {
            if let Some(builder) = $n.pop() {
                builder
            } else {
                CustomFlatBuffersBuilder::new()
            }
        }

        pub struct CustomFlatBuffersBuilder {
            pub builder: FlatBufferBuilder<'static>,
        }

        impl CustomFlatBuffersBuilder {
            fn new() -> Self {
                CustomFlatBuffersBuilder {
                    builder: FlatBufferBuilder::new(),
                }
            }

            const fn from_builder(builder: FlatBufferBuilder<'static>) -> Self {
                Self { builder }
            }

            #[allow(dead_code)]
            pub fn finish<T>(&mut self, root: WIPOffset<T>) -> &[u8] {
                self.builder.finish(root, None);

                self.builder.finished_data()
            }
        }

        impl Drop for CustomFlatBuffersBuilder {
            fn drop(&mut self) {
                if $n.len() < $l {
                    let mut builder = std::mem::take(&mut self.builder);
                    builder.reset();
                    let _ = $n.push(Self::from_builder(builder));
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_drop() {
        pool_flatbuffers!(100, BUILDERS, get_builder);
        let before = BUILDERS.len();

        {
            CustomFlatBuffersBuilder::new();
            CustomFlatBuffersBuilder::new();
        }

        assert_eq!(BUILDERS.len(), before + 2);

        {
            CustomFlatBuffersBuilder::new();
            CustomFlatBuffersBuilder::new();
        }

        assert_eq!(BUILDERS.len(), before + 4);
    }

    #[test]
    #[serial]
    fn test_get_builder() {
        pool_flatbuffers!(100, BUILDERS, get_builder);
        let _ = BUILDERS.push(CustomFlatBuffersBuilder::new());
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
