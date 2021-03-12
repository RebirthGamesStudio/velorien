use crate::uid::Uid;
use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage, NullStorage};
use specs_idvs::IdvStorage;
use vek::*;

// Position
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pos(pub Vec3<f32>);

impl Component for Pos {
    type Storage = IdvStorage<Self>;
}

// Velocity
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vel(pub Vec3<f32>);

impl Component for Vel {
    type Storage = IdvStorage<Self>;
}

/// Cache of Velocity (of last tick) * dt (of curent tick)
/// It's updated and read in physics sys to speed up entity<->entity collisions
/// no need to send it via network
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PreviousPhysCache {
    pub velocity_dt: Vec3<f32>,
    pub center: Vec3<f32>,
    /// Calculates a Sphere over the Entity for quick boundary checking
    pub collision_boundary: f32,
    pub scale: f32,
    pub scaled_radius: f32,
}

impl Component for PreviousPhysCache {
    type Storage = IdvStorage<Self>;
}

// Scale
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale(pub f32);

impl Component for Scale {
    type Storage = DerefFlaggedStorage<Self, IdvStorage<Self>>;
}

// Mass
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mass(pub f32);

impl Component for Mass {
    type Storage = DerefFlaggedStorage<Self, IdvStorage<Self>>;
}

// Collider
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Collider {
    // TODO: pass the map from ids -> voxel data to get_radius and get_z_limits to compute a
    // bounding cylinder
    Voxel { id: String },
    Box { radius: f32, z_min: f32, z_max: f32 },
    Point,
}

impl Collider {
    pub fn get_radius(&self) -> f32 {
        match self {
            Collider::Voxel { .. } => 1.0,
            Collider::Box { radius, .. } => *radius,
            Collider::Point => 0.0,
        }
    }

    pub fn get_z_limits(&self, modifier: f32) -> (f32, f32) {
        match self {
            Collider::Voxel { .. } => (0.0, 1.0),
            Collider::Box { z_min, z_max, .. } => (*z_min * modifier, *z_max * modifier),
            Collider::Point => (0.0, 0.0),
        }
    }
}

impl Component for Collider {
    type Storage = DerefFlaggedStorage<Self, IdvStorage<Self>>;
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gravity(pub f32);

impl Component for Gravity {
    type Storage = DerefFlaggedStorage<Self, IdvStorage<Self>>;
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sticky;

impl Component for Sticky {
    type Storage = DerefFlaggedStorage<Self, NullStorage<Self>>;
}

// PhysicsState
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsState {
    pub on_ground: bool,
    pub on_ceiling: bool,
    pub on_wall: Option<Vec3<f32>>,
    pub touch_entities: Vec<Uid>,
    pub in_liquid: Option<f32>, // Depth
    pub ground_vel: Vec3<f32>,
}

impl PhysicsState {
    pub fn reset(&mut self) {
        // Avoid allocation overhead!
        let mut touch_entities = std::mem::take(&mut self.touch_entities);
        touch_entities.clear();
        *self = Self {
            touch_entities,
            ground_vel: self.ground_vel, // Preserved, since it's the velocity of the last contact point
            ..Self::default()
        }
    }

    pub fn on_surface(&self) -> Option<Vec3<f32>> {
        self.on_ground
            .then_some(-Vec3::unit_z())
            .or_else(|| self.on_ceiling.then_some(Vec3::unit_z()))
            .or(self.on_wall)
    }
}

impl Component for PhysicsState {
    type Storage = IdvStorage<Self>;
}

// ForceUpdate
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ForceUpdate;

impl Component for ForceUpdate {
    type Storage = NullStorage<Self>;
}
