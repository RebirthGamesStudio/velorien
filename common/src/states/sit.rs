use super::utils::*;
use crate::{
    comp::{CharacterState, InventoryManip, StateUpdate},
    states::behavior::{CharacterBehavior, JoinData},
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Data;

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_wield(data, &mut update);
        handle_jump(&data, &mut update);

        // Try to Fall/Stand up/Move
        if !data.physics.on_ground || data.inputs.move_dir.magnitude_squared() > 0.0 {
            update.character = CharacterState::Idle;
        }

        update
    }

    fn wield(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);
        attempt_wield(data, &mut update);
        update
    }

    fn dance(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);
        attempt_dance(data, &mut update);
        update
    }

    fn stand(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);
        // Try to Fall/Stand up/Move
        update.character = CharacterState::Idle;
        update
    }

    fn modify_loadout(&self, data: &JoinData, inv_manip: InventoryManip) -> StateUpdate {
        let mut update = StateUpdate::from(data);
        handle_modify_loadout(&mut update, inv_manip);
        update
    }
}
