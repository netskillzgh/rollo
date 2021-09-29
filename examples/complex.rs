use async_trait::async_trait;
use rollo::{
    error::Error,
    game::interval_timer::{IntervalTimerExecutor, IntervalTimerMgr},
    packet::Packet,
    server::{
        dos_protection::DosPolicy,
        world::WorldI,
        world_session::{SocketTools, WorldSessionI},
        world_socket_mgr::{ListenerSecurity, WorldSocketMgr},
    },
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[tokio::main]
async fn main() {
    let world = Box::new(MyWorld {
        elapsed: AtomicU64::new(0),
        bg: BattlegroundManager {
            timer: IntervalTimerMgr::new(20),
        },
    });
    let world = Box::leak(world);

    let mut socket_manager = WorldSocketMgr::new(world);
    socket_manager
        .start_game_loop(15)
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

struct MyWorld {
    bg: BattlegroundManager,
    elapsed: AtomicU64,
}

impl WorldI for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
    fn update(&'static self, diff: i64) {
        self.bg.timer.update(diff, &self.bg);
    }

    fn time(&self) -> i64 {
        self.elapsed.load(Ordering::Acquire) as i64
    }

    fn update_time(&self, _new_time: i64) {}

    fn get_packet_limits(&self, _cmd: u16) -> (u16, u32, DosPolicy) {
        (10, 1024 * 10, DosPolicy::Log)
    }
}

struct MyWorldSession {
    socket_tools: SocketTools,
}

#[async_trait]
impl WorldSessionI<MyWorld> for MyWorldSession {
    async fn on_open(
        tools: SocketTools,
        _world: &'static MyWorld,
    ) -> Result<std::sync::Arc<Self>, Error> {
        println!("On Open");
        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }

    async fn on_dos_trigger(_world_session: &Arc<Self>, _world: &'static MyWorld, _cmd: u16) {}

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &'static MyWorld, _packet: Packet) {
        println!("Message");
    }

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

struct BattlegroundManager {
    timer: IntervalTimerMgr,
}

impl IntervalTimerExecutor for BattlegroundManager {
    fn on_update(&self, diff: i64) {
        println!("The diff is {}", diff);
    }
}