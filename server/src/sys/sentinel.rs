use super::SysTimer;
use common::{
    comp::{
        Auras, BeamSegment, Body, Buffs, CanBuild, CharacterState, Collider, Energy, Gravity,
        Group, Health, Item, LightEmitter, Loadout, Mass, MountState, Mounting, Ori, Player, Pos,
        Scale, Shockwave, Stats, Sticky, Vel,
    },
    msg::EcsCompPacket,
    span,
    sync::{CompSyncPackage, EntityPackage, EntitySyncPackage, Uid, UpdateTracker, WorldSyncExt},
};
use hashbrown::HashMap;
use specs::{
    shred::ResourceId, Entity as EcsEntity, Join, ReadExpect, ReadStorage, System, SystemData,
    World, Write, WriteExpect,
};
use vek::*;

/// Always watching
/// This system will monitor specific components for insertion, removal, and
/// modification
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, SysTimer<Self>>,
        TrackedComps<'a>,
        WriteTrackers<'a>,
    );

    fn run(&mut self, (mut timer, comps, mut trackers): Self::SystemData) {
        span!(_guard, "run", "sentinel::Sys::run");
        timer.start();

        record_changes(&comps, &mut trackers);

        timer.end();
    }
}

// Probably more difficult than it needs to be :p
#[derive(SystemData)]
pub struct TrackedComps<'a> {
    pub uid: ReadStorage<'a, Uid>,
    pub body: ReadStorage<'a, Body>,
    pub player: ReadStorage<'a, Player>,
    pub stats: ReadStorage<'a, Stats>,
    pub buffs: ReadStorage<'a, Buffs>,
    pub auras: ReadStorage<'a, Auras>,
    pub energy: ReadStorage<'a, Energy>,
    pub health: ReadStorage<'a, Health>,
    pub can_build: ReadStorage<'a, CanBuild>,
    pub light_emitter: ReadStorage<'a, LightEmitter>,
    pub item: ReadStorage<'a, Item>,
    pub scale: ReadStorage<'a, Scale>,
    pub mounting: ReadStorage<'a, Mounting>,
    pub mount_state: ReadStorage<'a, MountState>,
    pub group: ReadStorage<'a, Group>,
    pub mass: ReadStorage<'a, Mass>,
    pub collider: ReadStorage<'a, Collider>,
    pub sticky: ReadStorage<'a, Sticky>,
    pub gravity: ReadStorage<'a, Gravity>,
    pub loadout: ReadStorage<'a, Loadout>,
    pub character_state: ReadStorage<'a, CharacterState>,
    pub shockwave: ReadStorage<'a, Shockwave>,
    pub beam_segment: ReadStorage<'a, BeamSegment>,
}
impl<'a> TrackedComps<'a> {
    pub fn create_entity_package(
        &self,
        entity: EcsEntity,
        pos: Option<Pos>,
        vel: Option<Vel>,
        ori: Option<Ori>,
    ) -> EntityPackage<EcsCompPacket> {
        let uid = self
            .uid
            .get(entity)
            .copied()
            .expect("No uid to create an entity package")
            .0;
        let mut comps = Vec::new();
        self.body.get(entity).copied().map(|c| comps.push(c.into()));
        self.player
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.stats
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.buffs
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.auras
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.energy
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.health
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.can_build
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.light_emitter
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.item.get(entity).cloned().map(|c| comps.push(c.into()));
        self.scale
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.mounting
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.mount_state
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.group
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.mass.get(entity).copied().map(|c| comps.push(c.into()));
        self.collider
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.sticky
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.gravity
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.loadout
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.character_state
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.shockwave
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.beam_segment
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        // Add untracked comps
        pos.map(|c| comps.push(c.into()));
        vel.map(|c| comps.push(c.into()));
        ori.map(|c| comps.push(c.into()));

        EntityPackage { uid, comps }
    }
}
#[derive(SystemData)]
pub struct ReadTrackers<'a> {
    pub uid: ReadExpect<'a, UpdateTracker<Uid>>,
    pub body: ReadExpect<'a, UpdateTracker<Body>>,
    pub player: ReadExpect<'a, UpdateTracker<Player>>,
    pub stats: ReadExpect<'a, UpdateTracker<Stats>>,
    pub buffs: ReadExpect<'a, UpdateTracker<Buffs>>,
    pub auras: ReadExpect<'a, UpdateTracker<Auras>>,
    pub energy: ReadExpect<'a, UpdateTracker<Energy>>,
    pub health: ReadExpect<'a, UpdateTracker<Health>>,
    pub can_build: ReadExpect<'a, UpdateTracker<CanBuild>>,
    pub light_emitter: ReadExpect<'a, UpdateTracker<LightEmitter>>,
    pub item: ReadExpect<'a, UpdateTracker<Item>>,
    pub scale: ReadExpect<'a, UpdateTracker<Scale>>,
    pub mounting: ReadExpect<'a, UpdateTracker<Mounting>>,
    pub mount_state: ReadExpect<'a, UpdateTracker<MountState>>,
    pub group: ReadExpect<'a, UpdateTracker<Group>>,
    pub mass: ReadExpect<'a, UpdateTracker<Mass>>,
    pub collider: ReadExpect<'a, UpdateTracker<Collider>>,
    pub sticky: ReadExpect<'a, UpdateTracker<Sticky>>,
    pub gravity: ReadExpect<'a, UpdateTracker<Gravity>>,
    pub loadout: ReadExpect<'a, UpdateTracker<Loadout>>,
    pub character_state: ReadExpect<'a, UpdateTracker<CharacterState>>,
    pub shockwave: ReadExpect<'a, UpdateTracker<Shockwave>>,
    pub beam_segment: ReadExpect<'a, UpdateTracker<BeamSegment>>,
}
impl<'a> ReadTrackers<'a> {
    pub fn create_sync_packages(
        &self,
        comps: &TrackedComps,
        filter: impl Join + Copy,
        deleted_entities: Vec<u64>,
    ) -> (EntitySyncPackage, CompSyncPackage<EcsCompPacket>) {
        let entity_sync_package =
            EntitySyncPackage::new(&comps.uid, &self.uid, filter, deleted_entities);
        let comp_sync_package = CompSyncPackage::new()
            .with_component(&comps.uid, &*self.body, &comps.body, filter)
            .with_component(&comps.uid, &*self.player, &comps.player, filter)
            .with_component(&comps.uid, &*self.stats, &comps.stats, filter)
            .with_component(&comps.uid, &*self.buffs, &comps.buffs, filter)
            .with_component(&comps.uid, &*self.auras, &comps.auras, filter)
            .with_component(&comps.uid, &*self.energy, &comps.energy, filter)
            .with_component(&comps.uid, &*self.health, &comps.health, filter)
            .with_component(&comps.uid, &*self.can_build, &comps.can_build, filter)
            .with_component(
                &comps.uid,
                &*self.light_emitter,
                &comps.light_emitter,
                filter,
            )
            .with_component(&comps.uid, &*self.item, &comps.item, filter)
            .with_component(&comps.uid, &*self.scale, &comps.scale, filter)
            .with_component(&comps.uid, &*self.mounting, &comps.mounting, filter)
            .with_component(&comps.uid, &*self.mount_state, &comps.mount_state, filter)
            .with_component(&comps.uid, &*self.group, &comps.group, filter)
            .with_component(&comps.uid, &*self.mass, &comps.mass, filter)
            .with_component(&comps.uid, &*self.collider, &comps.collider, filter)
            .with_component(&comps.uid, &*self.sticky, &comps.sticky, filter)
            .with_component(&comps.uid, &*self.gravity, &comps.gravity, filter)
            .with_component(&comps.uid, &*self.loadout, &comps.loadout, filter)
            .with_component(
                &comps.uid,
                &*self.character_state,
                &comps.character_state,
                filter,
            )
            .with_component(&comps.uid, &*self.shockwave, &comps.shockwave, filter)
            .with_component(&comps.uid, &*self.beam_segment, &comps.beam_segment, filter);

        (entity_sync_package, comp_sync_package)
    }
}

#[derive(SystemData)]
pub struct WriteTrackers<'a> {
    uid: WriteExpect<'a, UpdateTracker<Uid>>,
    body: WriteExpect<'a, UpdateTracker<Body>>,
    player: WriteExpect<'a, UpdateTracker<Player>>,
    stats: WriteExpect<'a, UpdateTracker<Stats>>,
    buffs: WriteExpect<'a, UpdateTracker<Buffs>>,
    auras: WriteExpect<'a, UpdateTracker<Auras>>,
    energy: WriteExpect<'a, UpdateTracker<Energy>>,
    health: WriteExpect<'a, UpdateTracker<Health>>,
    can_build: WriteExpect<'a, UpdateTracker<CanBuild>>,
    light_emitter: WriteExpect<'a, UpdateTracker<LightEmitter>>,
    item: WriteExpect<'a, UpdateTracker<Item>>,
    scale: WriteExpect<'a, UpdateTracker<Scale>>,
    mounting: WriteExpect<'a, UpdateTracker<Mounting>>,
    mount_state: WriteExpect<'a, UpdateTracker<MountState>>,
    group: WriteExpect<'a, UpdateTracker<Group>>,
    mass: WriteExpect<'a, UpdateTracker<Mass>>,
    collider: WriteExpect<'a, UpdateTracker<Collider>>,
    sticky: WriteExpect<'a, UpdateTracker<Sticky>>,
    gravity: WriteExpect<'a, UpdateTracker<Gravity>>,
    loadout: WriteExpect<'a, UpdateTracker<Loadout>>,
    character_state: WriteExpect<'a, UpdateTracker<CharacterState>>,
    shockwave: WriteExpect<'a, UpdateTracker<Shockwave>>,
    beam: WriteExpect<'a, UpdateTracker<BeamSegment>>,
}

fn record_changes(comps: &TrackedComps, trackers: &mut WriteTrackers) {
    // Update trackers
    trackers.uid.record_changes(&comps.uid);
    trackers.body.record_changes(&comps.body);
    trackers.player.record_changes(&comps.player);
    trackers.stats.record_changes(&comps.stats);
    trackers.buffs.record_changes(&comps.buffs);
    trackers.auras.record_changes(&comps.auras);
    trackers.energy.record_changes(&comps.energy);
    trackers.health.record_changes(&comps.health);
    trackers.can_build.record_changes(&comps.can_build);
    trackers.light_emitter.record_changes(&comps.light_emitter);
    trackers.item.record_changes(&comps.item);
    trackers.scale.record_changes(&comps.scale);
    trackers.mounting.record_changes(&comps.mounting);
    trackers.mount_state.record_changes(&comps.mount_state);
    trackers.group.record_changes(&comps.group);
    trackers.mass.record_changes(&comps.mass);
    trackers.collider.record_changes(&comps.collider);
    trackers.sticky.record_changes(&comps.sticky);
    trackers.gravity.record_changes(&comps.gravity);
    trackers.loadout.record_changes(&comps.loadout);
    trackers
        .character_state
        .record_changes(&comps.character_state);
    trackers.shockwave.record_changes(&comps.shockwave);
    trackers.beam.record_changes(&comps.beam_segment);
    // Debug how many updates are being sent
    /*
    macro_rules! log_counts {
        ($comp:ident, $name:expr) => {
            // Note: if this will be used in actual server it would be more efficient to
            // count during record_changes
            let tracker = &trackers.$comp;
            let inserted = tracker.inserted().into_iter().count();
            let modified = tracker.modified().into_iter().count();
            let removed = tracker.removed().into_iter().count();
            tracing::warn!("{:6} insertions detected for    {}", inserted, $name);
            tracing::warn!("{:6} modifications detected for {}", modified, $name);
            tracing::warn!("{:6} deletions detected for     {}", removed, $name);
        };
    };
    log_counts!(uid, "Uids");
    log_counts!(body, "Bodies");
    log_counts!(buffs, "Buffs");
    log_counts!(auras, "Auras");
    log_counts!(player, "Players");
    log_counts!(stats, "Stats");
    log_counts!(energy, "Energies");
    log_vounts!(health, "Healths");
    log_counts!(light_emitter, "Light emitters");
    log_counts!(item, "Items");
    log_counts!(scale, "Scales");
    log_counts!(mounting, "Mountings");
    log_counts!(mount_state, "Mount States");
    log_counts!(mass, "Masses");
    log_counts!(collider, "Colliders");
    log_counts!(sticky, "Stickies");
    log_counts!(gravity, "Gravitys");
    log_counts!(loadout, "Loadouts");
    log_counts!(character_state, "Character States");
    log_counts!(shockwave, "Shockwaves");
    log_counts!(beam, "Beams");
    */
}

pub fn register_trackers(world: &mut World) {
    world.register_tracker::<Uid>();
    world.register_tracker::<Body>();
    world.register_tracker::<Player>();
    world.register_tracker::<Stats>();
    world.register_tracker::<Buffs>();
    world.register_tracker::<Auras>();
    world.register_tracker::<Energy>();
    world.register_tracker::<Health>();
    world.register_tracker::<CanBuild>();
    world.register_tracker::<LightEmitter>();
    world.register_tracker::<Item>();
    world.register_tracker::<Scale>();
    world.register_tracker::<Mounting>();
    world.register_tracker::<MountState>();
    world.register_tracker::<Group>();
    world.register_tracker::<Mass>();
    world.register_tracker::<Collider>();
    world.register_tracker::<Sticky>();
    world.register_tracker::<Gravity>();
    world.register_tracker::<Loadout>();
    world.register_tracker::<CharacterState>();
    world.register_tracker::<Shockwave>();
    world.register_tracker::<BeamSegment>();
}

/// Deleted entities grouped by region
pub struct DeletedEntities {
    map: HashMap<Vec2<i32>, Vec<u64>>,
}

impl Default for DeletedEntities {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl DeletedEntities {
    pub fn record_deleted_entity(&mut self, uid: Uid, region_key: Vec2<i32>) {
        self.map
            .entry(region_key)
            .or_insert(Vec::new())
            .push(uid.into());
    }

    pub fn take_deleted_in_region(&mut self, key: Vec2<i32>) -> Option<Vec<u64>> {
        self.map.remove(&key)
    }

    pub fn get_deleted_in_region(&mut self, key: Vec2<i32>) -> Option<&Vec<u64>> {
        self.map.get(&key)
    }

    pub fn take_remaining_deleted(&mut self) -> Vec<(Vec2<i32>, Vec<u64>)> {
        // TODO: don't allocate
        self.map.drain().collect()
    }
}
