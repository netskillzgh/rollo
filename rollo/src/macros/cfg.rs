#![allow(unused_macros)]

macro_rules! cfg_server {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "server"))]
            $item
        )*
    }
}

macro_rules! cfg_game {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "game"))]
            $item
        )*
    }
}

macro_rules! cfg_flat_buffers_helpers {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "flat_buffers_helpers"))]
            $item
        )*
    }
}

macro_rules! cfg_math {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "math"))]
            $item
        )*
    }
}
