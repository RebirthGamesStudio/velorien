use crate::{
    combat::GroupTarget,
    comp::{
        aura::{AuraBuffConstructor, AuraChange, AuraKind, AuraTarget, Specifier},
        CharacterState, StateUpdate,
    },
    event::ServerEvent,
    states::{
        behavior::{CharacterBehavior, JoinData},
        utils::*,
    },
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Separated out to condense update portions of character state
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticData {
    /// How long until state should create the aura
    pub buildup_duration: Duration,
    /// How long the state is creating an aura
    pub cast_duration: Duration,
    /// How long the state has until exiting
    pub recover_duration: Duration,
    /// Determines how the aura selects its targets
    pub targets: GroupTarget,
    /// Has information used to construct the aura
    pub aura: AuraBuffConstructor,
    /// How long aura lasts
    pub aura_duration: Duration,
    /// Radius of aura
    pub range: f32,
    /// What key is used to press ability
    pub ability_info: AbilityInfo,
    /// Whether the aura's effect scales with the user's current combo
    pub scales_with_combo: bool,
    /// Used to specify aura to the frontend
    pub specifier: Specifier,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Struct containing data that does not change over the course of the
    /// character state
    pub static_data: StaticData,
    /// Timer for each stage
    pub timer: Duration,
    /// What section the character stage is in
    pub stage_section: StageSection,
    /// Whether the aura has been cast already or not
    pub exhausted: bool,
    /// Combo which is updated only on the first tick behavior is called
    pub combo_at_cast: f32,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_orientation(data, &mut update, 1.0);
        handle_move(data, &mut update, 0.8);
        handle_jump(data, &mut update, 1.0);

        match self.stage_section {
            StageSection::Buildup => {
                if self.timer < self.static_data.buildup_duration {
                    let combo = if self.combo_at_cast > 0.0 {
                        self.combo_at_cast
                    } else {
                        if self.static_data.scales_with_combo {
                            update.server_events.push_front(ServerEvent::ComboChange {
                                entity: data.entity,
                                change: -(data.combo.counter() as i32),
                            });
                        }
                        data.combo.counter() as f32
                    };
                    // Build up
                    update.character = CharacterState::BasicAura(Data {
                        timer: tick_attack_or_default(data, self.timer, None),
                        combo_at_cast: combo,
                        ..*self
                    });
                } else if !self.exhausted {
                    // Creates aura if it hasn't been created already
                    let targets =
                        AuraTarget::from((Some(self.static_data.targets), Some(data.uid)));
                    let mut aura = self.static_data.aura.to_aura(
                        data.uid,
                        self.static_data.range,
                        Some(self.static_data.aura_duration),
                        targets,
                    );
                    if self.static_data.scales_with_combo {
                        match aura.aura_kind {
                            AuraKind::Buff {
                                kind: _,
                                ref mut data,
                                category: _,
                                source: _,
                            } => {
                                data.strength *= 1.0 + (self.combo_at_cast).log(2.0_f32);
                            },
                        }
                    }
                    update.server_events.push_front(ServerEvent::Aura {
                        entity: data.entity,
                        aura_change: AuraChange::Add(aura),
                    });
                    // Build up
                    update.character = CharacterState::BasicAura(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Cast,
                        exhausted: true,
                        ..*self
                    });
                }
            },
            StageSection::Cast => {
                if self.timer < self.static_data.cast_duration {
                    // Cast
                    update.character = CharacterState::BasicAura(Data {
                        timer: tick_attack_or_default(data, self.timer, None),
                        ..*self
                    });
                } else {
                    update.character = CharacterState::BasicAura(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Recover,
                        ..*self
                    });
                }
            },
            StageSection::Recover => {
                if self.timer < self.static_data.recover_duration {
                    update.character = CharacterState::BasicAura(Data {
                        timer: tick_attack_or_default(data, self.timer, None),
                        ..*self
                    });
                } else {
                    // Done
                    update.character = CharacterState::Wielding;
                }
            },
            _ => {
                // If it somehow ends up in an incorrect stage section
                update.character = CharacterState::Wielding;
            },
        }

        // At end of state logic so an interrupt isn't overwritten
        if !input_is_pressed(data, self.static_data.ability_info.input) {
            handle_state_interrupt(data, &mut update, false);
        }

        update
    }
}
