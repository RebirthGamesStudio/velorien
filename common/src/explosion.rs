use crate::{combat::GroupTarget, effect::Effect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Explosion {
    pub effects: Vec<RadiusEffect>,
    pub radius: f32,
    pub energy_regen: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RadiusEffect {
    TerrainDestruction(f32),
    Entity(Option<GroupTarget>, Effect),
}