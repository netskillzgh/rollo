use super::{dos_protection::DosPolicy, world_session::WorldSessionI};

pub trait WorldI: Sized + Sync {
    type WorldSessionimplementer: WorldSessionI<Self> + 'static + Send + Sync;

    cfg_game! {
        /// Tick
        fn update(&'static self, _diff: i64) {}
    }

    /// Dos Protection (Cmd, Size, Policy)
    fn get_packet_limits(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        (15, 10 * 2024, DosPolicy::Log)
    }
    /// Current Time
    fn time(&self) -> i64;
    /// Update the time
    fn update_time(&self, new_time: i64);
}