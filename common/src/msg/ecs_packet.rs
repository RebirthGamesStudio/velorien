use crate::{comp, sync};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use sum_type::sum_type;

// Automatically derive From<T> for EcsCompPacket
// for each variant EcsCompPacket::T(T.)
sum_type! {
    #[allow(clippy::large_enum_variant)] // TODO: Pending review in #587
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum EcsCompPacket {
        Body(comp::Body),
        Player(comp::Player),
        CanBuild(comp::CanBuild),
        Stats(comp::Stats),
        Buffs(comp::Buffs),
        Energy(comp::Energy),
        LightEmitter(comp::LightEmitter),
        Item(comp::Item),
        Scale(comp::Scale),
        Group(comp::Group),
        MountState(comp::MountState),
        Mounting(comp::Mounting),
        Mass(comp::Mass),
        Collider(comp::Collider),
        Gravity(comp::Gravity),
        Sticky(comp::Sticky),
        Loadout(comp::Loadout),
        CharacterState(comp::CharacterState),
        Pos(comp::Pos),
        Vel(comp::Vel),
        Ori(comp::Ori),
        Shockwave(comp::Shockwave),
        BeamSegment(comp::BeamSegment),
    }
}
// Automatically derive From<T> for EcsCompPhantom
// for each variant EcsCompPhantom::T(PhantomData<T>).
sum_type! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum EcsCompPhantom {
        Body(PhantomData<comp::Body>),
        Player(PhantomData<comp::Player>),
        CanBuild(PhantomData<comp::CanBuild>),
        Stats(PhantomData<comp::Stats>),
        Buffs(PhantomData<comp::Buffs>),
        Energy(PhantomData<comp::Energy>),
        LightEmitter(PhantomData<comp::LightEmitter>),
        Item(PhantomData<comp::Item>),
        Scale(PhantomData<comp::Scale>),
        Group(PhantomData<comp::Group>),
        MountState(PhantomData<comp::MountState>),
        Mounting(PhantomData<comp::Mounting>),
        Mass(PhantomData<comp::Mass>),
        Collider(PhantomData<comp::Collider>),
        Gravity(PhantomData<comp::Gravity>),
        Sticky(PhantomData<comp::Sticky>),
        Loadout(PhantomData<comp::Loadout>),
        CharacterState(PhantomData<comp::CharacterState>),
        Pos(PhantomData<comp::Pos>),
        Vel(PhantomData<comp::Vel>),
        Ori(PhantomData<comp::Ori>),
        Shockwave(PhantomData<comp::Shockwave>),
        BeamSegment(PhantomData<comp::BeamSegment>),
    }
}
impl sync::CompPacket for EcsCompPacket {
    type Phantom = EcsCompPhantom;

    fn apply_insert(self, entity: specs::Entity, world: &specs::World) {
        match self {
            EcsCompPacket::Body(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Player(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::CanBuild(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Stats(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Buffs(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Energy(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::LightEmitter(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Item(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Scale(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Group(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::MountState(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Mounting(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Mass(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Collider(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Gravity(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Sticky(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Loadout(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::CharacterState(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Pos(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Vel(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Ori(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::Shockwave(comp) => sync::handle_insert(comp, entity, world),
            EcsCompPacket::BeamSegment(comp) => sync::handle_insert(comp, entity, world),
        }
    }

    fn apply_modify(self, entity: specs::Entity, world: &specs::World) {
        match self {
            EcsCompPacket::Body(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Player(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::CanBuild(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Stats(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Buffs(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Energy(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::LightEmitter(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Item(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Scale(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Group(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::MountState(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Mounting(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Mass(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Collider(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Gravity(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Sticky(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Loadout(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::CharacterState(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Pos(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Vel(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Ori(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::Shockwave(comp) => sync::handle_modify(comp, entity, world),
            EcsCompPacket::BeamSegment(comp) => sync::handle_modify(comp, entity, world),
        }
    }

    fn apply_remove(phantom: Self::Phantom, entity: specs::Entity, world: &specs::World) {
        match phantom {
            EcsCompPhantom::Body(_) => sync::handle_remove::<comp::Body>(entity, world),
            EcsCompPhantom::Player(_) => sync::handle_remove::<comp::Player>(entity, world),
            EcsCompPhantom::CanBuild(_) => sync::handle_remove::<comp::CanBuild>(entity, world),
            EcsCompPhantom::Stats(_) => sync::handle_remove::<comp::Stats>(entity, world),
            EcsCompPhantom::Buffs(_) => sync::handle_remove::<comp::Buffs>(entity, world),
            EcsCompPhantom::Energy(_) => sync::handle_remove::<comp::Energy>(entity, world),
            EcsCompPhantom::LightEmitter(_) => {
                sync::handle_remove::<comp::LightEmitter>(entity, world)
            },
            EcsCompPhantom::Item(_) => sync::handle_remove::<comp::Item>(entity, world),
            EcsCompPhantom::Scale(_) => sync::handle_remove::<comp::Scale>(entity, world),
            EcsCompPhantom::Group(_) => sync::handle_remove::<comp::Group>(entity, world),
            EcsCompPhantom::MountState(_) => sync::handle_remove::<comp::MountState>(entity, world),
            EcsCompPhantom::Mounting(_) => sync::handle_remove::<comp::Mounting>(entity, world),
            EcsCompPhantom::Mass(_) => sync::handle_remove::<comp::Mass>(entity, world),
            EcsCompPhantom::Collider(_) => sync::handle_remove::<comp::Collider>(entity, world),
            EcsCompPhantom::Gravity(_) => sync::handle_remove::<comp::Gravity>(entity, world),
            EcsCompPhantom::Sticky(_) => sync::handle_remove::<comp::Sticky>(entity, world),
            EcsCompPhantom::Loadout(_) => sync::handle_remove::<comp::Loadout>(entity, world),
            EcsCompPhantom::CharacterState(_) => {
                sync::handle_remove::<comp::CharacterState>(entity, world)
            },
            EcsCompPhantom::Pos(_) => sync::handle_remove::<comp::Pos>(entity, world),
            EcsCompPhantom::Vel(_) => sync::handle_remove::<comp::Vel>(entity, world),
            EcsCompPhantom::Ori(_) => sync::handle_remove::<comp::Ori>(entity, world),
            EcsCompPhantom::Shockwave(_) => sync::handle_remove::<comp::Shockwave>(entity, world),
            EcsCompPhantom::BeamSegment(_) => sync::handle_remove::<comp::Ori>(entity, world),
        }
    }
}
