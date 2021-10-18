#![cfg_attr(docsrs, deny(rustdoc::broken_intra_doc_links))]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![doc = include_str!("../README.md")]

#[macro_use]
#[doc(hidden)]
pub(crate) mod macros;

cfg_server! {
    mod io;
}

#[doc(hidden)]
#[cfg(any(feature = "flatbuffers_helpers", feature = "game"))]
pub extern crate lazy_static;

pub mod error;

cfg_flatbuffers_helpers! {
    pub mod flatbuffers_helpers;
}

cfg_game! {
    pub mod game;
}

cfg_server! {
    pub mod packet;
    pub mod server;
}
