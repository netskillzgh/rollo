#![allow(unused_macros)]

macro_rules! cfg_server {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "server")]
            #[cfg_attr(docsrs, doc(cfg(feature = "server")))]
            $item
        )*
    }
}

macro_rules! cfg_macros {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "macros")]
            #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
            $item
        )*
    }
}

macro_rules! cfg_pointer_64 {
    ($($item:item)*) => {
        $(
            #[cfg(target_pointer_width = "64")]
            $item
        )*
    }
}

macro_rules! cfg_pointer_32 {
    ($($item:item)*) => {
        $(
            #[cfg(target_pointer_width = "32")]
            $item
        )*
    }
}

macro_rules! cfg_game {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "game")]
            #[cfg_attr(docsrs, doc(cfg(feature = "game")))]
            $item
        )*
    }
}

macro_rules! cfg_flatbuffers_helpers {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "flatbuffers_helpers")]
            #[cfg_attr(docsrs, doc(cfg(feature = "flatbuffers_helpers")))]
            $item
        )*
    }
}
