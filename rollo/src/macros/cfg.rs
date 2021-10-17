#![allow(unused_macros)]

macro_rules! cfg_server {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "server")]
            $item
        )*
    }
}

macro_rules! cfg_macros {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "macros")]
            $item
        )*
    }
}

macro_rules! cfg_game {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "game")]
            $item
        )*
    }
}

macro_rules! cfg_flatbuffers_helpers {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "flatbuffers_helpers")]
            $item
        )*
    }
}
