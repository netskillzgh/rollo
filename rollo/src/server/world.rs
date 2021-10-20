use super::{dos_protection::DosPolicy, world_session::WorldSession};

/// Events for the World
pub trait World: Sized + Sync + WorldTime + Send {
    type WorldSessionimplementer: WorldSession<Self> + 'static + Send + Sync;

    /// Tick
    fn update(&'static self, _diff: i64) {}

    /// Packet limit.
    /// (amount, size, policy)
    fn get_packet_limit(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        // 15 packets maximum per second.
        // 10 * 1024 bytes maximum per second.
        // Log if exceed.
        (15, 10 * 1024, DosPolicy::Log)
    }

    /// Limit for all packets per second.
    /// (amount of packets, size of packets)
    fn global_limit(&self) -> (u16, u32) {
        // 50 packets maximum per second.
        // 5000 bytes maximum per second.
        (50, 5000)
    }
}

/// Current World Time
pub trait WorldTime: Sized + Sync {
    /// Current Time
    fn time(&self) -> i64;
    /// Update the time
    fn update_time(&self, new_time: i64);
}

cfg_macros! {
    /// ## Implement WorldTime
    /// ### Examples
    /// ```rust, no_run
    /// use rollo::server::world_time;
    /// use std::sync::atomic::AtomicI64;
    ///
    /// let world = MyWorld {
    ///     time: AtomicI64::new(0)
    /// };
    ///
    /// let time = world.time();
    ///
    /// #[world_time]
    /// struct MyWorld {}
    /// ```
    pub use rollo_macros::world_time;
}
