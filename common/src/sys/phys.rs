use crate::{
    comp::{Body, Gravity, Mass, Mounting, Ori, PhysicsState, Pos, Scale, Sticky, Vel},
    event::{EventBus, ServerEvent},
    state::DeltaTime,
    sync::Uid,
    terrain::{Block, TerrainGrid},
    util::combination_to_pair_4,
    vol::ReadVol,
};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
use std::{
    f32,
    ops::{Div, Mul, Sub},
};
use vek::*;

pub const GRAVITY: f32 = 9.81 * 7.0;
const BOUYANCY: f32 = 0.0;
// Friction values used for linear damping. They are unitless quantities. The
// value of these quantities must be between zero and one. They represent the
// amount an object will slow down within 1/60th of a second. Eg. if the frction
// is 0.01, and the speed is 1.0, then after 1/60th of a second the speed will
// be 0.99. after 1 second the speed will be 0.54, which is 0.99 ^ 60.
const FRIC_GROUND: f32 = 0.15;
const FRIC_AIR: f32 = 0.0125;
const FRIC_FLUID: f32 = 0.2;

// Integrates forces, calculates the new velocity based off of the old velocity
// mass = entity mass
// dt = delta time
// lv = linear velocity
// damp = linear damping
// fluid = fluid force
// Friction is a type of damping.
fn integrate_forces(
    mass: f32,
    dt: f32,
    mut lv: Vec3<f32>,
    grav: f32,
    fluid: Option<Vec3<f32>>,
    damp: f32,
) -> Vec3<f32> {
    // this is not linear damping, because it is proportional to the original
    // velocity this "linear" damping in in fact, quite exponential. and thus
    // must be interpolated accordingly
    let linear_damp = (1.0 - damp.min(1.0)).powf(dt * 60.0);

    if let Some(fluid) = fluid {
        lv = (lv + ((fluid / mass - lv) * dt)).map(|e| e.max(-80.0));
        // lv = (lv + fluid / mass * dt).map(|e| e.max(-80.0));
    }
    lv.z = (lv.z - grav * dt).max(-80.0);
    lv * linear_damp
}

/// This system applies forces and calculates new positions and velocities.
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Uid>,
        ReadExpect<'a, TerrainGrid>,
        Read<'a, DeltaTime>,
        Read<'a, EventBus<ServerEvent>>,
        ReadStorage<'a, Scale>,
        ReadStorage<'a, Sticky>,
        ReadStorage<'a, Mass>,
        ReadStorage<'a, Gravity>,
        ReadStorage<'a, Body>,
        WriteStorage<'a, PhysicsState>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        WriteStorage<'a, Ori>,
        ReadStorage<'a, Mounting>,
    );

    fn run(
        &mut self,
        (
            entities,
            uids,
            terrain,
            dt,
            event_bus,
            scales,
            stickies,
            masses,
            gravities,
            bodies,
            mut physics_states,
            mut positions,
            mut velocities,
            mut orientations,
            mountings,
        ): Self::SystemData,
    ) {
        let mut event_emitter = event_bus.emitter();

        // Apply movement inputs
        for (entity, scale, sticky, mass, _b, mut pos, mut vel, _ori, _) in (
            &entities,
            scales.maybe(),
            stickies.maybe(),
            masses.maybe(),
            &bodies,
            &mut positions,
            &mut velocities,
            &mut orientations,
            !&mountings,
        )
            .join()
        {
            let mut physics_state = physics_states.get(entity).cloned().unwrap_or_default();

            if sticky.is_some() && (physics_state.on_wall.is_some() || physics_state.on_ground) {
                continue;
            }

            let scale = scale.map(|s| s.0).unwrap_or(1.0);

            // Basic collision with terrain
            // TODO: rename this, not just the player entity
            let player_rad = 0.3 * scale; // half-width of the player's AABB
            let player_height = 1.5 * scale;

            // Probe distances
            let hdist = player_rad.ceil() as i32;
            let vdist = player_height.ceil() as i32;
            // Neighbouring blocks iterator
            let near_iter = (-hdist..hdist + 1)
                .map(move |i| {
                    (-hdist..hdist + 1).map(move |j| (0..vdist + 1).map(move |k| (i, j, k)))
                })
                .flatten()
                .flatten();

            let old_vel = *vel;
            // Integrate forces
            // Friction is assumed to be a constant dependent on location
            let friction = FRIC_AIR
                .max(if physics_state.on_ground {
                    FRIC_GROUND
                } else {
                    0.0
                })
                .max(if physics_state.in_fluid.is_some() {
                    FRIC_FLUID
                } else {
                    0.0
                });
            let downward_force = if physics_state.in_fluid.is_some() {
                (1.0 - BOUYANCY) * GRAVITY
            } else {
                GRAVITY
            } * gravities.get(entity).map(|g| g.0).unwrap_or_default();
            let fluid_force = physics_state.in_fluid;
            let mass = mass.map(|m| m.0).unwrap_or(scale);
            vel.0 = integrate_forces(mass, dt.0, vel.0, downward_force, fluid_force, friction);

            // this is an approximation that allows most framerates to
            // behave in a similar manner.
            let vel_approx = (vel.0 + old_vel.0 * 4.0) * 0.2;
            // Don't move if we're not in a loaded chunk
            let pos_delta = if terrain
                .get_key(terrain.pos_key(pos.0.map(|e| e.floor() as i32)))
                .is_some()
            {
                vel_approx * dt.0
            } else {
                Vec3::zero()
            };

            // Function for determining whether the player at a specific position collides
            // with the ground
            fn collision_with_full<'a>(
                terrain: &ReadExpect<'a, TerrainGrid>,
                player_rad: f32,
                player_height: f32,
                pos: Vec3<f32>,
                hit: fn(&Block) -> bool,
                mut do_hit: impl FnMut(&Block) -> bool,
                near_iter: impl Iterator<Item = (i32, i32, i32)>,
            ) -> bool {
                for (i, j, k) in near_iter {
                    let block_pos = pos.map(|e| e.floor() as i32) + Vec3::new(i, j, k);

                    let vox = if let Ok(vox) = terrain.get(block_pos) {
                        vox
                    } else {
                        continue;
                    };
                    if hit(vox) {
                        let player_aabb = Aabb {
                            min: pos + Vec3::new(-player_rad, -player_rad, 0.0),
                            max: pos + Vec3::new(player_rad, player_rad, player_height),
                        };
                        let block_aabb = Aabb {
                            min: block_pos.map(|e| e as f32),
                            max: block_pos.map(|e| e as f32) + 1.0,
                        };

                        if player_aabb.collides_with_aabb(block_aabb) {
                            if do_hit(vox) {
                                return true;
                            }
                        }
                    }
                }
                false
            };

            let collision_with = |pos: Vec3<f32>, hit: fn(&Block) -> bool, near_iter| {
                collision_with_full(
                    &terrain,
                    player_rad,
                    player_height,
                    pos,
                    hit,
                    |_| true,
                    near_iter,
                )
            };

            let was_on_ground = physics_state.on_ground;
            physics_state.on_ground = false;

            let mut on_ground = false;
            let mut attempts = 0; // Don't loop infinitely here

            // Don't jump too far at once
            let increments = (pos_delta.map(|e| e.abs()).reduce_partial_max() / 0.3)
                .ceil()
                .max(1.0);
            let old_pos = pos.0;
            for _ in 0..increments as usize {
                pos.0 += pos_delta / increments;

                const MAX_ATTEMPTS: usize = 16;

                // While the player is colliding with the terrain...
                while collision_with(pos.0, |vox| vox.is_solid(), near_iter.clone())
                    && attempts < MAX_ATTEMPTS
                {
                    // Calculate the player's AABB
                    let player_aabb = Aabb {
                        min: pos.0 + Vec3::new(-player_rad, -player_rad, 0.0),
                        max: pos.0 + Vec3::new(player_rad, player_rad, player_height),
                    };

                    // Determine the block that we are colliding with most (based on minimum
                    // collision axis)
                    let (_block_pos, block_aabb) = near_iter
                        .clone()
                        // Calculate the block's position in world space
                        .map(|(i, j, k)| pos.0.map(|e| e.floor() as i32) + Vec3::new(i, j, k))
                        // Calculate the AABB of the block
                        .map(|block_pos| {
                            (
                                block_pos,
                                Aabb {
                                    min: block_pos.map(|e| e as f32),
                                    max: block_pos.map(|e| e as f32) + 1.0,
                                },
                            )
                        })
                        // Make sure the block is actually solid
                        .filter(|(block_pos, _)| {
                            terrain
                                .get(*block_pos)
                                .map(|vox| vox.is_solid())
                                .unwrap_or(false)
                        })
                        // Determine whether the block's AABB collides with the player's AABB
                        .filter(|(_, block_aabb)| block_aabb.collides_with_aabb(player_aabb))
                        // Find the maximum of the minimum collision axes (this bit is weird, trust me that it works)
                        .min_by_key(|(_, block_aabb)| {
                            ((block_aabb.center() - player_aabb.center() - Vec3::unit_z() * 0.5)
                                .map(|e| e.abs())
                                .sum()
                                * 1_000_000.0) as i32
                        })
                        .expect("Collision detected, but no colliding blocks found!");

                    // Find the intrusion vector of the collision
                    let dir = player_aabb.collision_vector_with_aabb(block_aabb);

                    // Determine an appropriate resolution vector (i.e: the minimum distance needed
                    // to push out of the block)
                    let max_axis = dir.map(|e| e.abs()).reduce_partial_min();
                    let resolve_dir = -dir.map(|e| {
                        if e.abs().to_bits() == max_axis.to_bits() {
                            e
                        } else {
                            0.0
                        }
                    });

                    // When the resolution direction is pointing upwards, we must be on the ground
                    if resolve_dir.z > 0.0 && vel.0.z <= 0.0 {
                        on_ground = true;

                        if !was_on_ground {
                            event_emitter.emit(ServerEvent::LandOnGround { entity, vel: vel.0 });
                        }
                    }

                    // When the resolution direction is non-vertical, we must be colliding with a
                    // wall If the space above is free...
                    if !collision_with(Vec3::new(pos.0.x, pos.0.y, (pos.0.z + 0.1).ceil()), |vox| vox.is_solid(), near_iter.clone())
                        // ...and we're being pushed out horizontally...
                        && resolve_dir.z == 0.0
                        // ...and the vertical resolution direction is sufficiently great...
                        && -dir.z > 0.1
                        // ...and we're falling/standing OR there is a block *directly* beneath our current origin (note: not hitbox)...
                        && (vel.0.z <= 0.0 || terrain
                            .get((pos.0 - Vec3::unit_z() * 0.1).map(|e| e.floor() as i32))
                            .map(|vox| vox.is_solid())
                            .unwrap_or(false))
                        // ...and there is a collision with a block beneath our current hitbox...
                        && collision_with(
                            old_pos + resolve_dir - Vec3::unit_z() * 1.05,
                            |vox| vox.is_solid(),
                            near_iter.clone(),
                        )
                    {
                        // ...block-hop!
                        pos.0.z = (pos.0.z + 0.1).ceil();
                        vel.0.z = 0.0;
                        on_ground = true;
                        break;
                    } else {
                        // Correct the velocity
                        vel.0 = vel.0.map2(
                            resolve_dir,
                            |e, d| if d * e.signum() < 0.0 { 0.0 } else { e },
                        );
                    }

                    // Resolve the collision normally
                    pos.0 += resolve_dir;

                    attempts += 1;
                }

                if attempts == MAX_ATTEMPTS {
                    pos.0 = old_pos;
                    break;
                }
            }

            if on_ground {
                physics_state.on_ground = true;
            // If the space below us is free, then "snap" to the ground
            } else if collision_with(
                pos.0 - Vec3::unit_z() * 1.05,
                |vox| vox.is_solid(),
                near_iter.clone(),
            ) && vel.0.z < 0.0
                && vel.0.z > -1.5
                && was_on_ground
                && !terrain
                    .get(
                        Vec3::new(pos.0.x, pos.0.y, (pos.0.z - 0.05).floor())
                            .map(|e| e.floor() as i32),
                    )
                    .map(|vox| vox.is_solid())
                    .unwrap_or(false)
            {
                pos.0.z = (pos.0.z - 0.05).floor();
                physics_state.on_ground = true;
            }

            let dirs = [
                Vec3::unit_x(),
                Vec3::unit_y(),
                -Vec3::unit_x(),
                -Vec3::unit_y(),
            ];

            if let (wall_dir, true) = dirs.iter().fold((Vec3::zero(), false), |(a, hit), dir| {
                if collision_with(pos.0 + *dir * 0.01, |vox| vox.is_solid(), near_iter.clone()) {
                    (a + dir, true)
                } else {
                    (a, hit)
                }
            }) {
                physics_state.on_wall = Some(wall_dir);
            } else {
                physics_state.on_wall = None;
            }

            // Figure out if we're in water
            let mut water_force = Vec3::zero();
            let mut in_fluid = false;
            collision_with_full(
                &terrain,
                player_rad,
                player_height,
                pos.0,
                |vox| vox.is_fluid(),
                |vox| {
                    // We're in water, but don't stop there...
                    in_fluid = true;
                    /* // Decode the fluid velocity.
                    let sub_height = water_height.sub(31.0 / 32.0).max(wposf.z).fract();
                    // sub_height is reinterpreted such that encoded 0-31 means from 1/32 to 1.
                    let encoded_sub_height = sub_height.mul(32.0) as u32;
                    let water_packed = water_packed | (encoded_sub_height << 14);
                    // let water = Rgb::new(60, 90, 190);
                    let water = Block::new(BlockKind::Water, Rgb::new(water_packed & 0xFF0000, water_packed & 0xFF00, water_packed & 0xFF)); */
                    let color = if let Some(color) = vox.get_color() {
                        color
                    } else {
                        return false;
                    };
                    let water_packed =
                        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
                    let encoded_sub_b_t = (water_packed >> 17) & 127;
                    // If the encoded value isn't a legal combination, we assume it means that the
                    // whole block is water.
                    let (encoded_sub_bottom, encoded_sub_top) =
                        combination_to_pair_4(encoded_sub_b_t as u8).unwrap_or((0, 16));
                    /* let encoded_sub_offset = (water_packed >> 20) & 15;
                    let encoded_sub_height = (water_packed >> 16) & 15; */
                    let encoded_velocity = (water_packed >> 10) & 127;
                    let encoded_angle_p = (water_packed >> 6) & 15;
                    let encoded_angle_h = water_packed & 63;
                    let sub_top = (encoded_sub_top as f32).div(16.0);
                    let sub_bottom = (encoded_sub_bottom as f32).div(16.0);
                    // let block_height = sub_height + sub_offset;
                    let block_height = sub_top - sub_bottom;
                    let velocity_magnitude = (encoded_velocity as f32).div(16.0);
                    let angle_h = (encoded_angle_h as f32)
                        .sub(31.5)
                        .div(31.5)
                        .mul(f32::consts::PI);
                    let angle_p = (encoded_angle_p as f32)
                        .div(15.0)
                        .mul(-f32::consts::FRAC_PI_2);
                    let velocity_direction = Vec3::new(angle_h.cos(), angle_h.sin(), angle_p.sin());
                    let velocity = velocity_direction * velocity_magnitude;
                    let water_density = 997.0;
                    // F = A * p * v^2 where p = density of water = 997.0,
                    // A = cross-sectional area = 1.0 * block_height, and v = velocity.
                    let mut block_force = 0.5
                        * block_height
                        * water_density
                        * velocity.map(|v| (v * v) * (v.signum()));
                    block_force.z = 0.0;
                    /* velocity.map2(vel_approx, |e, v| ((e - v) * (e - v)) * ((e - v).signum())) */
                    // let block_force = 0.5 * block_height * velocity.map2(vel_approx, |e, v| ((e -
                    // v) * (e - v)) * ((e - v).signum())); println!("Pos: {:?},
                    // velocity_magnitude: {:?}, sub_height: {:?}, sub_offset: {:?}, block_height:
                    // {:?}, velocity: {:?}, old velocity: {:?}, force: {:?}", vox,
                    // velocity_magnitude, sub_height, sub_offset, block_height, velocity,
                    // vel_approx, block_force);
                    water_force += block_force;
                    false
                },
                near_iter.clone(),
            );

            physics_state.in_fluid = if in_fluid { Some(water_force) } else { None };

            let _ = physics_states.insert(entity, physics_state);
        }

        // Apply pushback
        for (pos, scale, mass, vel, _, _, _, physics) in (
            &positions,
            scales.maybe(),
            masses.maybe(),
            &mut velocities,
            &bodies,
            !&mountings,
            stickies.maybe(),
            &mut physics_states,
        )
            .join()
            .filter(|(_, _, _, _, _, _, sticky, physics)| {
                sticky.is_none() || (physics.on_wall.is_none() && !physics.on_ground)
            })
        {
            physics.touch_entity = None;

            let scale = scale.map(|s| s.0).unwrap_or(1.0);
            let mass = mass.map(|m| m.0).unwrap_or(scale);

            for (other, pos_other, scale_other, mass_other, _, _) in (
                &uids,
                &positions,
                scales.maybe(),
                masses.maybe(),
                &bodies,
                !&mountings,
            )
                .join()
            {
                let scale_other = scale_other.map(|s| s.0).unwrap_or(1.0);

                let mass_other = mass_other.map(|m| m.0).unwrap_or(scale_other);
                if mass_other == 0.0 {
                    continue;
                }

                let diff = Vec2::<f32>::from(pos.0 - pos_other.0);

                let collision_dist = 0.95 * (scale + scale_other);

                if diff.magnitude_squared() > 0.0
                    && diff.magnitude_squared() < collision_dist.powf(2.0)
                    && pos.0.z + 1.6 * scale > pos_other.0.z
                    && pos.0.z < pos_other.0.z + 1.6 * scale_other
                {
                    let force = (collision_dist - diff.magnitude()) * 2.0 * mass_other
                        / (mass + mass_other);
                    vel.0 += Vec3::from(diff.normalized()) * force;
                    physics.touch_entity = Some(*other);
                }
            }
        }
    }
}
