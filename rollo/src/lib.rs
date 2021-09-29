#[macro_use]
#[doc(hidden)]
pub(crate) mod macros;
mod io;

pub mod error;
pub use bytes;

cfg_flat_buffers_helpers! {
    pub mod flat_buffers_helpers;
}

cfg_game! {
    pub mod game;
}

cfg_server! {
    pub mod packet;
    pub use async_trait::async_trait;
    pub mod server;
}

cfg_math! {
    pub mod math;
}
