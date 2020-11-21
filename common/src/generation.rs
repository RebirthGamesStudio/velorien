use crate::{
    comp::{self, humanoid, Alignment, Body, Item},
    npc::{self, NPC_NAMES},
};
use vek::*;

pub enum EntityTemplate {
    Traveller,
}

#[derive(Clone)]
pub struct EntityInfo {
    pub pos: Vec3<f32>,
    pub is_waypoint: bool, // Edge case, overrides everything else
    pub is_giant: bool,
    pub has_agency: bool,
    pub alignment: Alignment,
    pub body: Body,
    pub name: Option<String>,
    pub main_tool: Option<Item>,
    pub second_tool: Option<Item>,
    pub scale: f32,
    pub level: Option<u32>,
    pub loot_drop: Option<Item>,
}

impl EntityInfo {
    pub fn at(pos: Vec3<f32>) -> Self {
        Self {
            pos,
            is_waypoint: false,
            is_giant: false,
            has_agency: true,
            alignment: Alignment::Wild,
            body: Body::Humanoid(humanoid::Body::random()),
            name: None,
            main_tool: Some(Item::empty()),
            second_tool: Some(Item::empty()),
            scale: 1.0,
            level: None,
            loot_drop: None,
        }
    }

    pub fn do_if(mut self, cond: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if cond {
            self = f(self);
        }
        self
    }

    pub fn into_waypoint(mut self) -> Self {
        self.is_waypoint = true;
        self
    }

    pub fn into_giant(mut self) -> Self {
        self.is_giant = true;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_agency(mut self, agency: bool) -> Self {
        self.has_agency = agency;
        self
    }

    pub fn with_main_tool(mut self, main_tool: Item) -> Self {
        self.main_tool = Some(main_tool);
        self
    }

    pub fn with_second_tool(mut self, second_tool: Item) -> Self {
        self.second_tool = Some(second_tool);
        self
    }

    pub fn with_loot_drop(mut self, loot_drop: Item) -> Self {
        self.loot_drop = Some(loot_drop);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_automatic_name(mut self) -> Self {
        self.name = match &self.body {
            Body::Humanoid(body) => Some(get_npc_name(&NPC_NAMES.humanoid, body.species)),
            Body::QuadrupedMedium(body) => {
                Some(get_npc_name(&NPC_NAMES.quadruped_medium, body.species))
            },
            Body::BirdMedium(body) => Some(get_npc_name(&NPC_NAMES.bird_medium, body.species)),
            Body::Theropod(body) => Some(get_npc_name(&NPC_NAMES.theropod, body.species)),
            Body::QuadrupedSmall(body) => {
                Some(get_npc_name(&NPC_NAMES.quadruped_small, body.species))
            },
            Body::Dragon(body) => Some(get_npc_name(&NPC_NAMES.dragon, body.species)),
            Body::QuadrupedLow(body) => Some(get_npc_name(&NPC_NAMES.quadruped_low, body.species)),
            Body::Golem(body) => Some(get_npc_name(&NPC_NAMES.golem, body.species)),
            Body::BipedLarge(body) => Some(get_npc_name(&NPC_NAMES.biped_large, body.species)),
            _ => None,
        }
        .map(|s| {
            if self.is_giant {
                format!("Giant {}", s)
            } else {
                s.to_string()
            }
        });
        self
    }
}

#[derive(Default)]
pub struct ChunkSupplement {
    pub entities: Vec<EntityInfo>,
}

impl ChunkSupplement {
    pub fn add_entity(&mut self, entity: EntityInfo) { self.entities.push(entity); }
}

pub fn get_npc_name<
    'a,
    Species,
    SpeciesData: for<'b> core::ops::Index<&'b Species, Output = npc::SpeciesNames>,
>(
    body_data: &'a comp::BodyData<npc::BodyNames, SpeciesData>,
    species: Species,
) -> &'a str {
    &body_data.species[&species].generic
}
