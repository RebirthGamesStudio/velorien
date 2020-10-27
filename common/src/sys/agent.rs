use crate::{
    comp::{
        self,
        agent::Activity,
        group,
        group::Invite,
        item::{tool::ToolKind, ItemKind},
        Agent, Alignment, Body, ControlAction, ControlEvent, Controller, Energy, GroupManip,
        LightEmitter, Loadout, MountState, Ori, PhysicsState, Pos, Scale, Stats, UnresolvedChatMsg,
        Vel,
    },
    event::{EventBus, ServerEvent},
    metrics::SysMetrics,
    path::{Chaser, TraversalConfig},
    span,
    state::{DeltaTime, Time, TimeOfDay},
    sync::{Uid, UidAllocator},
    terrain::{Block, TerrainGrid},
    time::DayPeriod,
    util::Dir,
    vol::ReadVol,
};
use rand::{thread_rng, Rng};
use specs::{
    saveload::{Marker, MarkerAllocator},
    Entities, Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage,
};
use vek::*;

/// This system will allow NPCs to modify their controller
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        (
            Read<'a, UidAllocator>,
            Read<'a, Time>,
            Read<'a, DeltaTime>,
            Read<'a, group::GroupManager>,
        ),
        ReadExpect<'a, SysMetrics>,
        Write<'a, EventBus<ServerEvent>>,
        Entities<'a>,
        ReadStorage<'a, Energy>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Vel>,
        ReadStorage<'a, Ori>,
        ReadStorage<'a, Scale>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Loadout>,
        ReadStorage<'a, PhysicsState>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, group::Group>,
        ReadExpect<'a, TerrainGrid>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Body>,
        WriteStorage<'a, Agent>,
        WriteStorage<'a, Controller>,
        ReadStorage<'a, MountState>,
        ReadStorage<'a, Invite>,
        Read<'a, TimeOfDay>,
        ReadStorage<'a, LightEmitter>,
    );

    #[allow(clippy::or_fun_call)] // TODO: Pending review in #587
    fn run(
        &mut self,
        (
            (uid_allocator, time, dt, group_manager),
            sys_metrics,
            event_bus,
            entities,
            energies,
            positions,
            velocities,
            orientations,
            scales,
            stats,
            loadouts,
            physics_states,
            uids,
            groups,
            terrain,
            alignments,
            bodies,
            mut agents,
            mut controllers,
            mount_states,
            invites,
            time_of_day,
            light_emitter,
        ): Self::SystemData,
    ) {
        let start_time = std::time::Instant::now();
        span!(_guard, "run", "agent::Sys::run");
        for (
            entity,
            energy,
            pos,
            vel,
            ori,
            alignment,
            loadout,
            physics_state,
            body,
            uid,
            agent,
            controller,
            mount_state,
            group,
            light_emitter,
        ) in (
            &entities,
            &energies,
            &positions,
            &velocities,
            &orientations,
            alignments.maybe(),
            &loadouts,
            &physics_states,
            bodies.maybe(),
            &uids,
            &mut agents,
            &mut controllers,
            mount_states.maybe(),
            groups.maybe(),
            light_emitter.maybe(),
        )
            .join()
        {
            // Hack, replace with better system when groups are more sophisticated
            // Override alignment if in a group unless entity is owned already
            let alignment = if !matches!(alignment, Some(Alignment::Owned(_))) {
                group
                    .and_then(|g| group_manager.group_info(*g))
                    .and_then(|info| uids.get(info.leader))
                    .copied()
                    .map(Alignment::Owned)
                    .or(alignment.copied())
            } else {
                alignment.copied()
            };

            // Skip mounted entities
            if mount_state
                .map(|ms| *ms != MountState::Unmounted)
                .unwrap_or(false)
            {
                continue;
            }

            controller.reset();
            let mut event_emitter = event_bus.emitter();
            // Light lanterns at night
            // TODO Add a method to turn on NPC lanterns underground
            let lantern_equipped = loadout.lantern.as_ref().map_or(false, |item| {
                matches!(item.kind(), comp::item::ItemKind::Lantern(_))
            });
            let lantern_turned_on = light_emitter.is_some();
            let day_period = DayPeriod::from(time_of_day.0);
            // Only emit event for agents that have a lantern equipped
            if lantern_equipped {
                let mut rng = thread_rng();
                if day_period.is_dark() && !lantern_turned_on {
                    // Agents with turned off lanterns turn them on randomly once it's nighttime and
                    // keep them on
                    // Only emit event for agents that sill need to
                    // turn on their lantern
                    if let 0 = rng.gen_range(0, 1000) {
                        controller.events.push(ControlEvent::EnableLantern)
                    }
                } else if lantern_turned_on && day_period.is_light() {
                    // agents with turned on lanterns turn them off randomly once it's daytime and
                    // keep them off
                    if let 0 = rng.gen_range(0, 2000) {
                        controller.events.push(ControlEvent::DisableLantern)
                    }
                }
            };

            let mut inputs = &mut controller.inputs;

            // Default to looking in orientation direction (can be overridden below)
            inputs.look_dir = ori.0;

            const AVG_FOLLOW_DIST: f32 = 6.0;
            const MAX_FOLLOW_DIST: f32 = 12.0;
            const MAX_CHASE_DIST: f32 = 18.0;
            const LISTEN_DIST: f32 = 16.0;
            const SEARCH_DIST: f32 = 48.0;
            const SIGHT_DIST: f32 = 80.0;
            const MIN_ATTACK_DIST: f32 = 2.0;
            const MAX_FLEE_DIST: f32 = 20.0;

            let scale = scales.get(entity).map(|s| s.0).unwrap_or(1.0);

            // This controls how picky NPCs are about their pathfinding. Giants are larger
            // and so can afford to be less precise when trying to move around
            // the world (especially since they would otherwise get stuck on
            // obstacles that smaller entities would not).
            let node_tolerance = scale * 1.5;
            let slow_factor = body.map(|b| b.base_accel() / 250.0).unwrap_or(0.0).min(1.0);

            let mut do_idle = false;
            let mut choose_target = false;

            'activity: {
                match &mut agent.activity {
                    Activity::Idle(bearing) => {
                        *bearing += Vec2::new(
                            thread_rng().gen::<f32>() - 0.5,
                            thread_rng().gen::<f32>() - 0.5,
                        ) * 0.1
                            - *bearing * 0.003
                            - agent.patrol_origin.map_or(Vec2::zero(), |patrol_origin| {
                                (pos.0 - patrol_origin).xy() * 0.0002
                            });

                        // Stop if we're too close to a wall
                        *bearing *= 0.1
                            + if terrain
                                .ray(
                                    pos.0 + Vec3::unit_z(),
                                    pos.0
                                        + Vec3::from(*bearing)
                                            .try_normalized()
                                            .unwrap_or(Vec3::unit_y())
                                            * 5.0
                                        + Vec3::unit_z(),
                                )
                                .until(Block::is_solid)
                                .cast()
                                .1
                                .map_or(true, |b| b.is_none())
                            {
                                0.9
                            } else {
                                0.0
                            };

                        if bearing.magnitude_squared() > 0.5f32.powf(2.0) {
                            inputs.move_dir = *bearing * 0.65;
                        }

                        // Put away weapon
                        if thread_rng().gen::<f32>() < 0.005 {
                            controller.actions.push(ControlAction::Unwield);
                        }

                        // Sit
                        if thread_rng().gen::<f32>() < 0.0035 {
                            controller.actions.push(ControlAction::Sit);
                        }

                        // Sometimes try searching for new targets
                        if thread_rng().gen::<f32>() < 0.1 {
                            choose_target = true;
                        }
                    },
                    Activity::Follow { target, chaser } => {
                        if let (Some(tgt_pos), _tgt_stats) =
                            (positions.get(*target), stats.get(*target))
                        {
                            let dist = pos.0.distance(tgt_pos.0);
                            // Follow, or return to idle
                            if dist > AVG_FOLLOW_DIST {
                                if let Some((bearing, speed)) = chaser.chase(
                                    &*terrain,
                                    pos.0,
                                    vel.0,
                                    tgt_pos.0,
                                    TraversalConfig {
                                        node_tolerance,
                                        slow_factor,
                                        on_ground: physics_state.on_ground,
                                        min_tgt_dist: AVG_FOLLOW_DIST,
                                    },
                                ) {
                                    inputs.move_dir =
                                        bearing.xy().try_normalized().unwrap_or(Vec2::zero())
                                            * speed.min(0.2 + (dist - AVG_FOLLOW_DIST) / 8.0);
                                    inputs.jump.set_state(bearing.z > 1.5);
                                    inputs.swimup.set_state(bearing.z > 0.5);
                                    inputs.swimdown.set_state(bearing.z < 0.5);
                                }
                            } else {
                                do_idle = true;
                            }
                        } else {
                            do_idle = true;
                        }
                    },
                    Activity::Attack {
                        target,
                        chaser,
                        been_close,
                        powerup,
                        ..
                    } => {
                        #[derive(Eq, PartialEq)]
                        enum Tactic {
                            Melee,
                            Axe,
                            Hammer,
                            Sword,
                            RangedPowerup,
                            Staff,
                            StoneGolemBoss,
                        }

                        let tactic = match loadout.active_item.as_ref().and_then(|ic| {
                            if let ItemKind::Tool(tool) = &ic.item.kind() {
                                Some(&tool.kind)
                            } else {
                                None
                            }
                        }) {
                            Some(ToolKind::Bow(_)) => Tactic::RangedPowerup,
                            Some(ToolKind::Staff(_)) => Tactic::Staff,
                            Some(ToolKind::Hammer(_)) => Tactic::Hammer,
                            Some(ToolKind::Sword(_)) => Tactic::Sword,
                            Some(ToolKind::Axe(_)) => Tactic::Axe,
                            Some(ToolKind::NpcWeapon(kind)) => match kind.as_str() {
                                "StoneGolemsFist" => Tactic::StoneGolemBoss,
                                _ => Tactic::Melee,
                            },
                            _ => Tactic::Melee,
                        };

                        if let (Some(tgt_pos), Some(tgt_stats), tgt_alignment) = (
                            positions.get(*target),
                            stats.get(*target),
                            alignments.get(*target).copied().unwrap_or(
                                uids.get(*target)
                                    .copied()
                                    .map(Alignment::Owned)
                                    .unwrap_or(Alignment::Wild),
                            ),
                        ) {
                            if let Some(dir) = Dir::from_unnormalized(tgt_pos.0 - pos.0) {
                                inputs.look_dir = dir;
                            }

                            // Don't attack entities we are passive towards
                            // TODO: This is here, it's a bit of a hack
                            if let Some(alignment) = alignment {
                                if alignment.passive_towards(tgt_alignment) || tgt_stats.is_dead {
                                    do_idle = true;
                                    break 'activity;
                                }
                            }

                            let dist_sqrd = pos.0.distance_squared(tgt_pos.0);

                            let damage = stats
                                .get(entity)
                                .map(|s| s.health.current() as f32 / s.health.maximum() as f32)
                                .unwrap_or(0.5);

                            // Flee
                            let flees = alignment
                                .map(|a| !matches!(a, Alignment::Enemy | Alignment::Owned(_)))
                                .unwrap_or(true);
                            if 1.0 - agent.psyche.aggro > damage && flees {
                                if dist_sqrd < MAX_FLEE_DIST.powf(2.0) {
                                    if let Some((bearing, speed)) = chaser.chase(
                                        &*terrain,
                                        pos.0,
                                        vel.0,
                                        // Away from the target (ironically)
                                        pos.0
                                            + (pos.0 - tgt_pos.0)
                                                .try_normalized()
                                                .unwrap_or_else(Vec3::unit_y)
                                                * 8.0,
                                        TraversalConfig {
                                            node_tolerance,
                                            slow_factor,
                                            on_ground: physics_state.on_ground,
                                            min_tgt_dist: 1.25,
                                        },
                                    ) {
                                        inputs.move_dir =
                                            bearing.xy().try_normalized().unwrap_or(Vec2::zero())
                                                * speed
                                                * 0.2; //Let small/slow animals flee slower than the player
                                        inputs.jump.set_state(bearing.z > 1.5);
                                        inputs.swimup.set_state(bearing.z > 0.5);
                                        inputs.swimdown.set_state(bearing.z < 0.5);
                                    }
                                } else {
                                    do_idle = true;
                                }
                            } else if (tactic == Tactic::Staff
                                && dist_sqrd < (5.0 * MIN_ATTACK_DIST * scale).powf(2.0))
                                || dist_sqrd < (MIN_ATTACK_DIST * scale).powf(2.0)
                            {
                                // Close-range attack
                                inputs.move_dir = (tgt_pos.0 - pos.0)
                                    .xy()
                                    .try_normalized()
                                    .unwrap_or(Vec2::unit_y())
                                    * 0.1;

                                match tactic {
                                    Tactic::Sword
                                    | Tactic::Melee
                                    | Tactic::Hammer
                                    | Tactic::StoneGolemBoss => inputs.primary.set_state(true),
                                    Tactic::Staff => {
                                        // Kind of arbitrary values, but feel right in game
                                        if energy.current() > 800 && thread_rng().gen::<f32>() > 0.8
                                        {
                                            inputs.ability3.set_state(true)
                                        } else if energy.current() > 10 {
                                            inputs.secondary.set_state(true)
                                        } else {
                                            inputs.primary.set_state(true)
                                        }
                                    },
                                    Tactic::Axe => {
                                        if *powerup > 6.0 {
                                            inputs.secondary.set_state(false);
                                            *powerup = 0.0;
                                        } else if *powerup > 4.0 && energy.current() > 10 {
                                            inputs.secondary.set_state(true);
                                            *powerup += dt.0;
                                        } else {
                                            inputs.primary.set_state(true);
                                            *powerup += dt.0;
                                        }
                                    },
                                    Tactic::RangedPowerup => inputs.roll.set_state(true),
                                }
                            } else if dist_sqrd < MAX_CHASE_DIST.powf(2.0)
                                || (dist_sqrd < SIGHT_DIST.powf(2.0)
                                    && (!*been_close || !matches!(tactic, Tactic::Melee)))
                            {
                                let can_see_tgt = terrain
                                    .ray(pos.0 + Vec3::unit_z(), tgt_pos.0 + Vec3::unit_z())
                                    .until(Block::is_opaque)
                                    .cast()
                                    .0
                                    .powf(2.0)
                                    >= dist_sqrd;

                                if can_see_tgt {
                                    if let Tactic::RangedPowerup = tactic {
                                        if *powerup > 1.5 {
                                            inputs.primary.set_state(false);
                                            *powerup = 0.0;
                                        } else {
                                            inputs.primary.set_state(true);
                                            *powerup += dt.0;
                                        }
                                    } else if let Tactic::Sword = tactic {
                                        if *powerup > 4.0 {
                                            inputs.secondary.set_state(true);
                                            *powerup = 0.0;
                                        } else {
                                            *powerup += dt.0;
                                        }
                                    } else if let Tactic::Staff = tactic {
                                        inputs.primary.set_state(true);
                                    } else if let Tactic::Hammer = tactic {
                                        if *powerup > 5.0 {
                                            inputs.ability3.set_state(true);
                                            *powerup = 0.0;
                                        } else {
                                            *powerup += dt.0;
                                        }
                                    } else if let Tactic::StoneGolemBoss = tactic {
                                        if *powerup > 5.0 {
                                            inputs.secondary.set_state(true);
                                            *powerup = 0.0;
                                        } else {
                                            *powerup += dt.0;
                                        }
                                    }
                                }

                                if dist_sqrd < MAX_CHASE_DIST.powf(2.0) {
                                    *been_close = true;
                                }

                                // Long-range chase
                                if let Some((bearing, speed)) = chaser.chase(
                                    &*terrain,
                                    pos.0,
                                    vel.0,
                                    tgt_pos.0,
                                    TraversalConfig {
                                        node_tolerance,
                                        slow_factor,
                                        on_ground: physics_state.on_ground,
                                        min_tgt_dist: 1.25,
                                    },
                                ) {
                                    if can_see_tgt {
                                        match tactic {
                                            Tactic::RangedPowerup => {
                                                inputs.move_dir = bearing
                                                    .xy()
                                                    .rotated_z(thread_rng().gen_range(0.5, 1.57))
                                                    .try_normalized()
                                                    .unwrap_or(Vec2::zero())
                                                    * speed;
                                            },
                                            Tactic::Staff => {
                                                inputs.move_dir = bearing
                                                    .xy()
                                                    .rotated_z(thread_rng().gen_range(-1.57, -0.5))
                                                    .try_normalized()
                                                    .unwrap_or(Vec2::zero())
                                                    * speed;
                                            },
                                            _ => {
                                                inputs.move_dir = bearing
                                                    .xy()
                                                    .try_normalized()
                                                    .unwrap_or(Vec2::zero())
                                                    * speed;
                                            },
                                        }
                                    } else {
                                        inputs.move_dir =
                                            bearing.xy().try_normalized().unwrap_or(Vec2::zero())
                                                * speed;
                                        inputs.jump.set_state(bearing.z > 1.5);
                                        inputs.swimup.set_state(bearing.z > 0.5);
                                        inputs.swimdown.set_state(bearing.z < 0.5);
                                    }
                                }

                                if dist_sqrd < 16.0f32.powf(2.0)
                                    && matches!(tactic, Tactic::Melee)
                                    && thread_rng().gen::<f32>() < 0.02
                                {
                                    inputs.roll.set_state(true);
                                }
                            } else {
                                do_idle = true;
                            }
                        } else {
                            do_idle = true;
                        }
                    },
                }
            }

            if do_idle {
                agent.activity = Activity::Idle(Vec2::zero());
            }

            // Choose a new target to attack: only go out of our way to attack targets we
            // are hostile toward!
            if choose_target {
                // Search for new targets (this looks expensive, but it's only run occasionally)
                // TODO: Replace this with a better system that doesn't consider *all* entities
                let closest_entity = (&entities, &positions, &stats, alignments.maybe())
                    .join()
                    .filter(|(e, e_pos, e_stats, e_alignment)| {
                        ((e_pos.0.distance_squared(pos.0) < SEARCH_DIST.powf(2.0) &&
                            // Within our view
                            (e_pos.0 - pos.0).try_normalized().map(|v| v.dot(*inputs.look_dir) > 0.15).unwrap_or(true))
                                // Within listen distance
                                || e_pos.0.distance_squared(pos.0) < LISTEN_DIST.powf(2.0))
                            && *e != entity
                            && !e_stats.is_dead
                            && alignment
                                .and_then(|a| e_alignment.map(|b| a.hostile_towards(*b)))
                                .unwrap_or(false)
                    })
                    // Can we even see them?
                    .filter(|(_, e_pos, _, _)| terrain
                        .ray(pos.0 + Vec3::unit_z(), e_pos.0 + Vec3::unit_z())
                        .until(Block::is_opaque)
                        .cast()
                        .0 >= e_pos.0.distance(pos.0))
                    .min_by_key(|(_, e_pos, _, _)| (e_pos.0.distance_squared(pos.0) * 100.0) as i32)
                    .map(|(e, _, _, _)| e);

                if let Some(target) = closest_entity {
                    agent.activity = Activity::Attack {
                        target,
                        chaser: Chaser::default(),
                        time: time.0,
                        been_close: false,
                        powerup: 0.0,
                    };
                }
            }

            // --- Activity overrides (in reverse order of priority: most important goes
            // last!) ---

            // Attack a target that's attacking us
            if let Some(my_stats) = stats.get(entity) {
                // Only if the attack was recent
                if my_stats.health.last_change.0 < 3.0 {
                    if let comp::HealthSource::Attack { by }
                    | comp::HealthSource::Projectile { owner: Some(by) }
                    | comp::HealthSource::Energy { owner: Some(by) }
                    | comp::HealthSource::Buff { owner: Some(by) }
                    | comp::HealthSource::Explosion { owner: Some(by) } =
                        my_stats.health.last_change.1.cause
                    {
                        if !agent.activity.is_attack() {
                            if let Some(attacker) = uid_allocator.retrieve_entity_internal(by.id())
                            {
                                if stats.get(attacker).map_or(false, |a| !a.is_dead) {
                                    match agent.activity {
                                        Activity::Attack { target, .. } if target == attacker => {},
                                        _ => {
                                            if agent.can_speak {
                                                let msg =
                                                    "npc.speech.villager_under_attack".to_string();
                                                event_emitter.emit(ServerEvent::Chat(
                                                    UnresolvedChatMsg::npc(*uid, msg),
                                                ));
                                            }

                                            agent.activity = Activity::Attack {
                                                target: attacker,
                                                chaser: Chaser::default(),
                                                time: time.0,
                                                been_close: false,
                                                powerup: 0.0,
                                            };
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Follow owner if we're too far, or if they're under attack
            if let Some(Alignment::Owned(owner)) = alignment {
                (|| {
                    let owner = uid_allocator.retrieve_entity_internal(owner.id())?;

                    let owner_pos = positions.get(owner)?;
                    let dist_sqrd = pos.0.distance_squared(owner_pos.0);
                    if dist_sqrd > MAX_FOLLOW_DIST.powf(2.0) && !agent.activity.is_follow() {
                        agent.activity = Activity::Follow {
                            target: owner,
                            chaser: Chaser::default(),
                        };
                    }

                    // Attack owner's attacker
                    let owner_stats = stats.get(owner)?;
                    if owner_stats.health.last_change.0 < 5.0
                        && owner_stats.health.last_change.1.amount < 0
                    {
                        if let comp::HealthSource::Attack { by } =
                            owner_stats.health.last_change.1.cause
                        {
                            if !agent.activity.is_attack() {
                                let attacker = uid_allocator.retrieve_entity_internal(by.id())?;

                                agent.activity = Activity::Attack {
                                    target: attacker,
                                    chaser: Chaser::default(),
                                    time: time.0,
                                    been_close: false,
                                    powerup: 0.0,
                                };
                            }
                        }
                    }

                    Some(())
                })();
            }

            debug_assert!(inputs.move_dir.map(|e| !e.is_nan()).reduce_and());
            debug_assert!(inputs.look_dir.map(|e| !e.is_nan()).reduce_and());
        }

        // Process group invites
        for (_invite, /*alignment,*/ agent, controller) in
            (&invites, /*&alignments,*/ &mut agents, &mut controllers).join()
        {
            let accept = false; // set back to "matches!(alignment, Alignment::Npc)" when we got better NPC recruitment mechanics
            if accept {
                // Clear agent comp
                *agent = Agent::default();
                controller
                    .events
                    .push(ControlEvent::GroupManip(GroupManip::Accept));
            } else {
                controller
                    .events
                    .push(ControlEvent::GroupManip(GroupManip::Decline));
            }
        }
        sys_metrics.agent_ns.store(
            start_time.elapsed().as_nanos() as i64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
