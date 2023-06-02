use super::{dos_protection::DosPolicy, world_session::WorldSession};
use crate::game::GameTime;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;

/// A trait defining the behavior of a game world.
///
/// This trait defines several methods that can be implemented by types that represent a game world.
/// The methods are used to handle events related to the world, such as when the server starts, when
/// the game time is updated, and when packets are sent and received.
#[async_trait]
pub trait World: Sized + Sync + Send {
    /// The type that implements the `WorldSession` trait for this world.
    type WorldSessionimplementer: WorldSession<Self> + 'static + Send + Sync;

    /// Called when the server starts.
    ///
    /// This method is called when the server starts. It takes a reference to a `GameTime` atomic cell
    /// as an argument, but doesn't return anything. It is an asynchronous method, as indicated by the
    /// `async_trait` attribute.
    async fn on_start(_game_time: &'static AtomicCell<GameTime>) {}

    /// Returns a reference to the game time atomic cell.
    ///
    /// This method returns an optional reference to a `GameTime` atomic cell. If the world doesn't use
    /// a game time, it should return `None`.
    fn game_time(&'static self) -> Option<&'static AtomicCell<GameTime>> {
        None
    }

    /// Called when the game time is updated.
    ///
    /// This method is called when the game time is updated. It takes a `diff` value of type `i64` and
    /// a `GameTime` value as arguments, but doesn't return anything.
    fn update(&'static self, _diff: i64, _game_time: GameTime) {}

    /// Returns the packet limit for a given command.
    ///
    /// This method takes a `cmd` value of type `u16` as an argument and returns a tuple containing the
    /// maximum amount of packets, the maximum size of packets, and a `DosPolicy` value indicating what
    /// to do if the limit is exceeded.
    fn get_packet_limit(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        // Default packet limit: 15 packets maximum per second, 10 * 1024 bytes maximum per second,
        // and log if the limit is exceeded.
        (15, 10 * 1024, DosPolicy::Log)
    }

    /// Returns the global packet limit per second.
    ///
    /// This method returns a tuple containing the maximum amount of packets and the maximum size of
    /// packets that can be sent globally per second.
    fn global_limit(&self) -> (u16, u32) {
        // Default global packet limit: 50 packets maximum per second and 5000 bytes maximum per second.
        (50, 5000)
    }
}