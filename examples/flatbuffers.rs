/* use rollo::{
    error::Error,
    flatbuffers_helpers::flatbuffers,
    packet::Packet,
    server::{ListenerSecurity, SocketTools, World, WorldSession, WorldSocketMgr},
    tokio,
};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let world = Box::leak(Box::new(MyWorld {}));

    // Get builder from the pool.
    let builder = rollo::flatbuffers_helpers::FLAT_BUFFER_BUILDER_GENERATOR.create();
    drop(builder);
    // Builder returned.

    let mut socket_manager = WorldSocketMgr::new(world);
    socket_manager
        .start_game_loop(Duration::from_millis(15))
        .start_network("127.0.0.1:6666", ListenerSecurity::Tcp)
        .await
        .unwrap();
}

struct MyWorld {}

impl World for MyWorld {
    type WorldSessionimplementer = MyWorldSession;
}

struct MyWorldSession {
    socket_tools: SocketTools,
}

#[rollo::async_trait]
impl WorldSession<MyWorld> for MyWorldSession {
    async fn on_open(
        tools: SocketTools,
        _world: &'static MyWorld,
    ) -> Result<std::sync::Arc<Self>, Error> {
        Ok(Arc::new(Self {
            socket_tools: tools,
        }))
    }

    fn socket_tools(&self) -> &SocketTools {
        &self.socket_tools
    }

    async fn on_message(_world_session: &Arc<Self>, _world: &'static MyWorld, _packet: Packet) {}

    async fn on_close(_world_session: &Arc<Self>, _world: &'static MyWorld) {}
}

// -- flatbuffers --
pub enum WeaponOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct Weapon<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Weapon<'a> {
    type Inner = Weapon<'a>;

    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf, loc },
        }
    }
}

impl<'a> Weapon<'a> {
    pub const VT_NAME: flatbuffers::VOffsetT = 4;
    pub const VT_DAMAGE: flatbuffers::VOffsetT = 6;

    pub const fn get_fully_qualified_name() -> &'static str {
        "MyGame.Sample.Weapon"
    }

    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Weapon { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args WeaponArgs<'args>,
    ) -> flatbuffers::WIPOffset<Weapon<'bldr>> {
        let mut builder = WeaponBuilder::new(_fbb);
        if let Some(x) = args.name {
            builder.add_name(x);
        }
        builder.add_damage(args.damage);
        builder.finish()
    }

    pub fn unpack(&self) -> WeaponT {
        let name = self.name().map(|x| x.to_string());
        let damage = self.damage();
        WeaponT { name, damage }
    }

    #[inline]
    pub fn name(&self) -> Option<&'a str> {
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Weapon::VT_NAME, None)
        }
    }
    #[inline]
    pub fn damage(&self) -> i16 {
        unsafe { self._tab.get::<i16>(Weapon::VT_DAMAGE, Some(0)).unwrap() }
    }
}

impl flatbuffers::Verifiable for Weapon<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("name", Self::VT_NAME, false)?
            .visit_field::<i16>("damage", Self::VT_DAMAGE, false)?
            .finish();
        Ok(())
    }
}
pub struct WeaponArgs<'a> {
    pub name: Option<flatbuffers::WIPOffset<&'a str>>,
    pub damage: i16,
}
impl<'a> Default for WeaponArgs<'a> {
    #[inline]
    fn default() -> Self {
        WeaponArgs {
            name: None,
            damage: 0,
        }
    }
}
pub struct WeaponBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> WeaponBuilder<'a, 'b> {
    #[inline]
    pub fn add_name(&mut self, name: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(Weapon::VT_NAME, name);
    }
    #[inline]
    pub fn add_damage(&mut self, damage: i16) {
        self.fbb_.push_slot::<i16>(Weapon::VT_DAMAGE, damage, 0);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> WeaponBuilder<'a, 'b> {
        let start = _fbb.start_table();
        WeaponBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<Weapon<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl std::fmt::Debug for Weapon<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("Weapon");
        ds.field("name", &self.name());
        ds.field("damage", &self.damage());
        ds.finish()
    }
}
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct WeaponT {
    pub name: Option<String>,
    pub damage: i16,
}
impl Default for WeaponT {
    fn default() -> Self {
        Self {
            name: None,
            damage: 0,
        }
    }
}
impl WeaponT {
    pub fn pack<'b>(
        &self,
        _fbb: &mut flatbuffers::FlatBufferBuilder<'b>,
    ) -> flatbuffers::WIPOffset<Weapon<'b>> {
        let name = self.name.as_ref().map(|x| _fbb.create_string(x));
        let damage = self.damage;
        Weapon::create(_fbb, &WeaponArgs { name, damage })
    }
}
 */

fn main() {
    assert!(true);
}
