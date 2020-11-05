use serde::{Deserialize, Serialize};
use specs::{Component, FlaggedStorage};
use specs_idvs::IdvStorage;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Energy {
    current: u32,
    maximum: u32,
    pub regen_rate: f32,
    pub last_change: Option<(i32, f64, EnergySource)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EnergySource {
    Ability,
    Climb,
    LevelUp,
    HitEnemy,
    Regen,
    Revive,
    Unknown,
}

#[derive(Debug)]
pub enum StatChangeError {
    Underflow,
    Overflow,
}

impl Energy {
    pub fn new(amount: u32) -> Energy {
        Energy {
            current: amount,
            maximum: amount,
            regen_rate: 0.0,
            last_change: None,
        }
    }

    pub fn current(&self) -> u32 { self.current }

    pub fn maximum(&self) -> u32 { self.maximum }

    pub fn set_to(&mut self, amount: u32, cause: EnergySource) {
        let amount = amount.min(self.maximum);
        self.last_change = Some((amount as i32 - self.current as i32, 0.0, cause));
        self.current = amount;
    }

    pub fn change_by(&mut self, change: EnergyChange) {
        self.current = ((self.current as i32 + change.amount).max(0) as u32).min(self.maximum);
        self.last_change = Some((change.amount, 0.0, change.source));
    }

    pub fn try_change_by(
        &mut self,
        amount: i32,
        cause: EnergySource,
    ) -> Result<(), StatChangeError> {
        if self.current as i32 + amount < 0 {
            Err(StatChangeError::Underflow)
        } else if self.current as i32 + amount > self.maximum as i32 {
            Err(StatChangeError::Overflow)
        } else {
            self.change_by(EnergyChange {
                amount,
                source: cause,
            });
            Ok(())
        }
    }

    pub fn set_maximum(&mut self, amount: u32) {
        self.maximum = amount;
        self.current = self.current.min(self.maximum);
    }
}

pub struct EnergyChange {
    pub amount: i32,
    pub source: EnergySource,
}

impl Component for Energy {
    type Storage = FlaggedStorage<Self, IdvStorage<Self>>;
}
