use crossbeam::atomic::AtomicCell;

use super::{dos_protection::DosPolicy, world_session::WorldSession};
use crate::game::GameTime;
use async_trait::async_trait;

/// Events for the World
#[async_trait]
pub trait World: Sized + Sync + Send {
    type WorldSessionimplementer: WorldSession<Self> + 'static + Send + Sync;

    async fn on_start(_game_time: &'static AtomicCell<GameTime>) {}
    fn game_time(&'static self) -> Option<&'static AtomicCell<GameTime>> {
        None
    }

    /// Update
    fn update(&'static self, _diff: i64, _game_time: GameTime) {}

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
