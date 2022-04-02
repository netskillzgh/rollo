mod event_processor;
pub use event_processor::{Event, EventProcessor};

pub(crate) mod game_loop;
pub use game_loop::GameLoop;

pub(crate) mod game_time;
pub use game_time::GameTime;

mod interval_mgr;
pub use interval_mgr::{IntervalExecutor, IntervalMgr};
