use crate::{
    comp::{CharacterState, StateUpdate},
    states::utils::*,
    sys::character_behavior::{CharacterBehavior, JoinData},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Separated out to condense update portions of character state
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticData {
    pub movement_duration: Duration,
    pub only_up: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Struct containing data that does not change over the course of the
    /// character state
    pub static_data: StaticData,
    /// Timer for each stage
    pub timer: Duration,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_move(data, &mut update, 1.0);

        if self.timer < self.static_data.movement_duration {
            // Movement
            if self.static_data.only_up {
                update.vel.0.z += 500.0 * data.dt.0;
            } else {
                update.vel.0 += *data.inputs.look_dir * 500.0 * data.dt.0;
            }
            update.character = CharacterState::Boost(Data {
                static_data: self.static_data,
                timer: self
                    .timer
                    .checked_add(Duration::from_secs_f32(data.dt.0))
                    .unwrap_or_default(),
            });
        } else {
            // Done
            update.character = CharacterState::Wielding;
        }

        update
    }
}
