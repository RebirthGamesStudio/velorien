use common::{
    combat,
    comp::{
        self,
        skills::{GeneralSkill, Skill},
        Body, CharacterState, Combo, Energy, EnergyChange, EnergySource, Health, Inventory, Poise,
        PoiseChange, PoiseSource, Pos, SkillSet, Stats,
    },
    event::{EventBus, ServerEvent},
    outcome::Outcome,
    resources::{DeltaTime, EntitiesDiedLastTick, Time},
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::HashSet;
use specs::{
    shred::ResourceId, Entities, Join, Read, ReadStorage, SystemData, World, Write, WriteStorage,
};
use vek::Vec3;

const ENERGY_REGEN_ACCEL: f32 = 10.0;
const POISE_REGEN_ACCEL: f32 = 2.0;

#[derive(SystemData)]
pub struct ReadData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    time: Read<'a, Time>,
    server_bus: Read<'a, EventBus<ServerEvent>>,
    positions: ReadStorage<'a, Pos>,
    uids: ReadStorage<'a, Uid>,
    bodies: ReadStorage<'a, Body>,
    char_states: ReadStorage<'a, CharacterState>,
    inventories: ReadStorage<'a, Inventory>,
}

/// This system kills players, levels them up, and regenerates energy.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadData<'a>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, SkillSet>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Poise>,
        WriteStorage<'a, Energy>,
        WriteStorage<'a, Combo>,
        Write<'a, EntitiesDiedLastTick>,
        Write<'a, Vec<Outcome>>,
    );

    const NAME: &'static str = "stats";
    const ORIGIN: Origin = Origin::Common;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            read_data,
            stats,
            mut skill_sets,
            mut healths,
            mut poises,
            mut energies,
            mut combos,
            mut entities_died_last_tick,
            mut outcomes,
        ): Self::SystemData,
    ) {
        entities_died_last_tick.0.clear();
        let mut server_event_emitter = read_data.server_bus.emitter();
        let dt = read_data.dt.0;

        // Increment last change timer
        healths.set_event_emission(false); // avoid unnecessary syncing
        poises.set_event_emission(false); // avoid unnecessary syncing
        for mut health in (&mut healths).join() {
            health.last_change.0 += f64::from(dt);
        }
        for mut poise in (&mut poises).join() {
            poise.last_change.0 += f64::from(dt);
        }
        healths.set_event_emission(true);
        poises.set_event_emission(true);

        // Update stats
        for (entity, uid, stats, mut skill_set, mut health, pos, mut energy, inventory) in (
            &read_data.entities,
            &read_data.uids,
            &stats,
            &mut skill_sets,
            &mut healths,
            &read_data.positions,
            &mut energies,
            read_data.inventories.maybe(),
        )
            .join()
        {
            let set_dead = { health.should_die() && !health.is_dead };

            if set_dead {
                let cloned_entity = (entity, *pos);
                entities_died_last_tick.0.push(cloned_entity);
                server_event_emitter.emit(ServerEvent::Destroy {
                    entity,
                    cause: health.last_change.1.cause,
                });

                health.is_dead = true;
            }
            let stat = stats;

            let update_max_hp = {
                (stat.max_health_modifier - 1.0).abs() > f32::EPSILON
                    || health.base_max() != health.maximum()
            };

            if update_max_hp {
                health.scale_maximum(stat.max_health_modifier);
            }

            let (change_energy, energy_scaling) = {
                // Calculates energy scaling from stats and inventory
                let new_energy_scaling =
                    combat::compute_max_energy_mod(&energy, inventory) + stat.max_energy_modifier;
                let current_energy_scaling = energy.maximum() as f32 / energy.base_max() as f32;
                // Only changes energy if new modifier different from old modifer
                // TODO: Look into using wider threshold incase floating point imprecision makes
                // this always true
                (
                    (current_energy_scaling - new_energy_scaling).abs() > f32::EPSILON,
                    new_energy_scaling,
                )
            };

            // If modifier sufficiently different, mutably access energy
            if change_energy {
                energy.scale_maximum(energy_scaling);
            }

            let skills_to_level = skill_set
                .skill_groups
                .iter()
                .filter_map(|s_g| {
                    (s_g.exp >= skill_set.skill_point_cost(s_g.skill_group_kind))
                        .then(|| s_g.skill_group_kind)
                })
                .collect::<HashSet<_>>();

            if !skills_to_level.is_empty() {
                for skill_group in skills_to_level {
                    skill_set.earn_skill_point(skill_group);
                    outcomes.push(Outcome::SkillPointGain {
                        uid: *uid,
                        skill_tree: skill_group,
                        total_points: skill_set.earned_sp(skill_group),
                        pos: pos.0,
                    });
                }
            }
        }

        // Apply effects from leveling skills
        for (mut skill_set, mut health, mut energy, body) in (
            &mut skill_sets,
            &mut healths,
            &mut energies,
            &read_data.bodies,
        )
            .join()
        {
            if skill_set.modify_health {
                let health_level = skill_set
                    .skill_level(Skill::General(GeneralSkill::HealthIncrease))
                    .unwrap_or(None)
                    .unwrap_or(0);
                health.update_max_hp(Some(*body), health_level);
                skill_set.modify_health = false;
            }
            if skill_set.modify_energy {
                let energy_level = skill_set
                    .skill_level(Skill::General(GeneralSkill::EnergyIncrease))
                    .unwrap_or(None)
                    .unwrap_or(0);
                energy.update_max_energy(Some(*body), energy_level);
                skill_set.modify_energy = false;
            }
        }

        // Update energies and poises
        for (character_state, mut energy, mut poise) in
            (&read_data.char_states, &mut energies, &mut poises).join()
        {
            match character_state {
                // Accelerate recharging energy.
                CharacterState::Idle { .. }
                | CharacterState::Talk { .. }
                | CharacterState::Sit { .. }
                | CharacterState::Dance { .. }
                | CharacterState::Sneak { .. }
                | CharacterState::Glide { .. }
                | CharacterState::GlideWield { .. }
                | CharacterState::Wielding { .. }
                | CharacterState::Equipping { .. }
                | CharacterState::Boost { .. } => {
                    let res = { energy.current() < energy.maximum() };

                    if res {
                        let energy = &mut *energy;
                        // Have to account for Calc I differential equations due to acceleration
                        energy.change_by(EnergyChange {
                            amount: (energy.regen_rate * dt + ENERGY_REGEN_ACCEL * dt.powi(2) / 2.0)
                                as i32,
                            source: EnergySource::Regen,
                        });
                        energy.regen_rate =
                            (energy.regen_rate + ENERGY_REGEN_ACCEL * dt).min(100.0);
                    }

                    let res_poise = { poise.current() < poise.maximum() };

                    if res_poise {
                        let poise = &mut *poise;
                        poise.change_by(
                            PoiseChange {
                                amount: (poise.regen_rate * dt
                                    + POISE_REGEN_ACCEL * dt.powi(2) / 2.0)
                                    as i32,
                                source: PoiseSource::Regen,
                            },
                            Vec3::zero(),
                        );
                        poise.regen_rate = (poise.regen_rate + POISE_REGEN_ACCEL * dt).min(10.0);
                    }
                },
                // Ability use does not regen and sets the rate back to zero.
                CharacterState::BasicMelee { .. }
                | CharacterState::DashMelee { .. }
                | CharacterState::LeapMelee { .. }
                | CharacterState::SpinMelee { .. }
                | CharacterState::ComboMelee { .. }
                | CharacterState::BasicRanged { .. }
                | CharacterState::ChargedMelee { .. }
                | CharacterState::ChargedRanged { .. }
                | CharacterState::RepeaterRanged { .. }
                | CharacterState::Shockwave { .. }
                | CharacterState::BasicBeam { .. }
                | CharacterState::BasicAura { .. }
                | CharacterState::Blink { .. }
                | CharacterState::BasicSummon { .. }
                | CharacterState::SelfBuff { .. }
                | CharacterState::SpriteSummon { .. } => {
                    if energy.regen_rate != 0.0 {
                        energy.regen_rate = 0.0
                    }
                },
                // Abilities that temporarily stall energy gain, but preserve regen_rate.
                CharacterState::Roll { .. }
                | CharacterState::Climb { .. }
                | CharacterState::Stunned { .. }
                | CharacterState::BasicBlock { .. }
                | CharacterState::UseItem { .. } => {},
            }
        }

        // Decay combo
        for (_, mut combo) in (&read_data.entities, &mut combos).join() {
            if combo.counter() > 0
                && read_data.time.0 - combo.last_increase() > comp::combo::COMBO_DECAY_START
            {
                combo.reset();
            }
        }
    }
}
