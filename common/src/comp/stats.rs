use crate::{
    comp,
    comp::{body::humanoid::Species, skills::SkillSet, Body},
    sync::Uid,
};
use serde::{Deserialize, Serialize};
use specs::{Component, FlaggedStorage};
use specs_idvs::IdvStorage;
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthChange {
    pub amount: i32,
    pub cause: HealthSource,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthSource {
    Attack { by: Uid }, // TODO: Implement weapon
    Projectile { owner: Option<Uid> },
    Explosion { owner: Option<Uid> },
    Energy { owner: Option<Uid> },
    Buff { owner: Option<Uid> },
    Suicide,
    World,
    Revive,
    Command,
    LevelUp,
    Item,
    Healing { by: Option<Uid> },
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Health {
    current: u32,
    maximum: u32,
    pub last_change: (f64, HealthChange),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Exp {
    current: u32,
    maximum: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Level {
    amount: u32,
}

impl Health {
    pub fn current(&self) -> u32 { self.current }

    pub fn maximum(&self) -> u32 { self.maximum }

    pub fn set_to(&mut self, amount: u32, cause: HealthSource) {
        let amount = amount.min(self.maximum);
        self.last_change = (0.0, HealthChange {
            amount: amount as i32 - self.current as i32,
            cause,
        });
        self.current = amount;
    }

    pub fn change_by(&mut self, change: HealthChange) {
        self.current = ((self.current as i32 + change.amount).max(0) as u32).min(self.maximum);
        self.last_change = (0.0, change);
    }

    // This is private because max hp is based on the level
    fn set_maximum(&mut self, amount: u32) {
        self.maximum = amount;
        self.current = self.current.min(self.maximum);
    }
}
#[derive(Debug)]
pub enum StatChangeError {
    Underflow,
    Overflow,
}

impl fmt::Display for StatChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Underflow => "insufficient stat quantity",
            Self::Overflow => "stat quantity would overflow",
        })
    }
}
impl Error for StatChangeError {}

impl Exp {
    /// Used to determine how much exp is required to reach the next level. When
    /// a character levels up, the next level target is increased by this value
    const EXP_INCREASE_FACTOR: u32 = 25;

    pub fn current(&self) -> u32 { self.current }

    pub fn maximum(&self) -> u32 { self.maximum }

    pub fn set_current(&mut self, current: u32) { self.current = current; }

    pub fn change_by(&mut self, current: i64) {
        self.current = ((self.current as i64) + current) as u32;
    }

    pub fn change_maximum_by(&mut self, maximum: i64) {
        self.maximum = ((self.maximum as i64) + maximum) as u32;
    }

    pub fn update_maximum(&mut self, level: u32) {
        self.maximum = level
            .saturating_mul(Self::EXP_INCREASE_FACTOR)
            .saturating_add(Self::EXP_INCREASE_FACTOR);
    }
}

impl Level {
    pub fn set_level(&mut self, level: u32) { self.amount = level; }

    pub fn level(&self) -> u32 { self.amount }

    pub fn change_by(&mut self, level: u32) { self.amount += level; }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub name: String,
    pub health: Health,
    pub level: Level,
    pub exp: Exp,
    pub skill_set: SkillSet,
    pub endurance: u32,
    pub fitness: u32,
    pub willpower: u32,
    pub is_dead: bool,
    pub body_type: Body,
}

impl Stats {
    pub fn should_die(&self) -> bool { self.health.current == 0 }

    pub fn revive(&mut self) {
        self.health
            .set_to(self.health.maximum(), HealthSource::Revive);
        self.is_dead = false;
    }

    // TODO: Delete this once stat points will be a thing
    pub fn update_max_hp(&mut self, body: Body) {
        self.health
            .set_maximum(body.base_health() + body.base_health_increase() * self.level.amount);
    }
}

impl Stats {
    pub fn new(name: String, body: Body) -> Self {
        let species = if let comp::Body::Humanoid(hbody) = body {
            Some(hbody.species)
        } else {
            None
        };

        // TODO: define base stats somewhere else (maybe method on Body?)
        let (endurance, fitness, willpower) = match species {
            Some(Species::Danari) => (0, 2, 3), // Small, flexible, intelligent, physically weak
            Some(Species::Dwarf) => (2, 2, 1),  // physically strong, intelligent, slow reflexes
            Some(Species::Elf) => (1, 2, 2),    // Intelligent, quick, physically weak
            Some(Species::Human) => (2, 1, 2),  // Perfectly balanced
            Some(Species::Orc) => (3, 2, 0),    /* Physically strong, non intelligent, medium */
            // reflexes
            Some(Species::Undead) => (1, 3, 1), /* Very good reflexes, equally intelligent and */
            // strong
            None => (0, 0, 0),
        };

        let mut stats = Self {
            name,
            health: Health {
                current: 0,
                maximum: 0,
                last_change: (0.0, HealthChange {
                    amount: 0,
                    cause: HealthSource::Revive,
                }),
            },
            level: Level { amount: 1 },
            exp: Exp {
                current: 0,
                maximum: 50,
            },
            skill_set: SkillSet::default(),
            endurance,
            fitness,
            willpower,
            is_dead: false,
            body_type: body,
        };

        stats.update_max_hp(body);
        stats
            .health
            .set_to(stats.health.maximum(), HealthSource::Revive);

        stats
    }

    /// Creates an empty `Stats` instance - used during character loading from
    /// the database
    pub fn empty() -> Self {
        Self {
            name: "".to_owned(),
            health: Health {
                current: 0,
                maximum: 0,
                last_change: (0.0, HealthChange {
                    amount: 0,
                    cause: HealthSource::Revive,
                }),
            },
            level: Level { amount: 1 },
            exp: Exp {
                current: 0,
                maximum: 50,
            },
            skill_set: SkillSet::default(),
            endurance: 0,
            fitness: 0,
            willpower: 0,
            is_dead: false,
            body_type: comp::Body::Humanoid(comp::body::humanoid::Body::random()),
        }
    }

    pub fn with_max_health(mut self, amount: u32) -> Self {
        self.health.maximum = amount;
        self.health.current = amount;
        self
    }
}

impl Component for Stats {
    type Storage = FlaggedStorage<Self, IdvStorage<Self>>;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Dying {
    pub cause: HealthSource,
}

impl Component for Dying {
    type Storage = IdvStorage<Self>;
}
