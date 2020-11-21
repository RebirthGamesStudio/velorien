use crate::{
    all::ForestKind,
    sim::{local_cells, Cave, Path, RiverKind, SimChunk, WorldSim},
    util::Sampler,
    IndexRef, CONFIG,
};
use common::{
    terrain::{
        quadratic_nearest_point, river_spline_coeffs, uniform_idx_as_vec2, vec2_as_uniform_idx,
        TerrainChunkSize,
    },
    vol::RectVolSize,
};
use noise::NoiseFn;
use serde::Deserialize;
use std::{
    cmp::Reverse,
    f32, f64,
    ops::{Add, Div, Mul, Sub},
};
use tracing::error;
use vek::*;

pub struct ColumnGen<'a> {
    pub sim: &'a WorldSim,
}

#[derive(Deserialize)]
pub struct Colors {
    pub cold_grass: (f32, f32, f32),
    pub warm_grass: (f32, f32, f32),
    pub dark_grass: (f32, f32, f32),
    pub wet_grass: (f32, f32, f32),
    pub cold_stone: (f32, f32, f32),
    pub hot_stone: (f32, f32, f32),
    pub warm_stone: (f32, f32, f32),
    pub beach_sand: (f32, f32, f32),
    pub desert_sand: (f32, f32, f32),
    pub snow: (f32, f32, f32),
    pub snow_moss: (f32, f32, f32),

    pub stone_col: (u8, u8, u8),

    pub dirt_low: (f32, f32, f32),
    pub dirt_high: (f32, f32, f32),

    pub snow_high: (f32, f32, f32),
    pub warm_stone_high: (f32, f32, f32),

    pub grass_high: (f32, f32, f32),
    pub tropical_high: (f32, f32, f32),
}

impl<'a> ColumnGen<'a> {
    pub fn new(sim: &'a WorldSim) -> Self { Self { sim } }
}

impl<'a> Sampler<'a> for ColumnGen<'a> {
    type Index = (Vec2<i32>, IndexRef<'a>);
    type Sample = Option<ColumnSample<'a>>;

    #[allow(clippy::float_cmp)] // TODO: Pending review in #587
    #[allow(clippy::if_same_then_else)] // TODO: Pending review in #587
    #[allow(clippy::nonminimal_bool)] // TODO: Pending review in #587
    #[allow(clippy::single_match)] // TODO: Pending review in #587
    fn get(&self, (wpos, index): Self::Index) -> Option<ColumnSample<'a>> {
        let wposf = wpos.map(|e| e as f64);
        let chunk_pos = wpos.map2(TerrainChunkSize::RECT_SIZE, |e, sz: u32| e / sz as i32);

        let sim = &self.sim;

        let _turb = Vec2::new(
            sim.gen_ctx.turb_x_nz.get((wposf.div(48.0)).into_array()) as f32,
            sim.gen_ctx.turb_y_nz.get((wposf.div(48.0)).into_array()) as f32,
        ) * 12.0;
        let wposf_turb = wposf; // + turb.map(|e| e as f64);

        let chaos = sim.get_interpolated(wpos, |chunk| chunk.chaos)?;
        let temp = sim.get_interpolated(wpos, |chunk| chunk.temp)?;
        let humidity = sim.get_interpolated(wpos, |chunk| chunk.humidity)?;
        let rockiness = sim.get_interpolated(wpos, |chunk| chunk.rockiness)?;
        let tree_density = sim.get_interpolated(wpos, |chunk| chunk.tree_density)?;
        let spawn_rate = sim.get_interpolated(wpos, |chunk| chunk.spawn_rate)?;
        let alt = sim.get_interpolated_monotone(wpos, |chunk| chunk.alt)?;
        let surface_veg = sim.get_interpolated_monotone(wpos, |chunk| chunk.surface_veg)?;
        let chunk_warp_factor = sim.get_interpolated_monotone(wpos, |chunk| chunk.warp_factor)?;
        let sim_chunk = sim.get(chunk_pos)?;
        let neighbor_coef = TerrainChunkSize::RECT_SIZE.map(|e| e as f64);
        let my_chunk_idx = vec2_as_uniform_idx(self.sim.map_size_lg(), chunk_pos);
        let neighbor_river_data =
            local_cells(self.sim.map_size_lg(), my_chunk_idx).filter_map(|neighbor_idx: usize| {
                let neighbor_pos = uniform_idx_as_vec2(self.sim.map_size_lg(), neighbor_idx);
                let neighbor_chunk = sim.get(neighbor_pos)?;
                Some((neighbor_pos, neighbor_chunk, &neighbor_chunk.river))
            });
        let lake_width = (TerrainChunkSize::RECT_SIZE.x as f64 * (2.0f64.sqrt())) + 12.0;
        let neighbor_river_data = neighbor_river_data.map(|(posj, chunkj, river)| {
            let kind = match river.river_kind {
                Some(kind) => kind,
                None => {
                    return (posj, chunkj, river, None);
                },
            };
            let downhill_pos = if let Some(pos) = chunkj.downhill {
                pos
            } else {
                match kind {
                    RiverKind::River { .. } => {
                        error!(?river, ?posj, "What?");
                        panic!("How can a river have no downhill?");
                    },
                    RiverKind::Lake { .. } => {
                        return (posj, chunkj, river, None);
                    },
                    RiverKind::Ocean => posj,
                }
            };
            let downhill_wpos = downhill_pos.map(|e| e as f64);
            let downhill_pos =
                downhill_pos.map2(TerrainChunkSize::RECT_SIZE, |e, sz: u32| e / sz as i32);
            let neighbor_pos = posj.map(|e| e as f64) * neighbor_coef;
            let direction = neighbor_pos - downhill_wpos;
            let river_width_min = if let RiverKind::River { cross_section } = kind {
                cross_section.x as f64
            } else {
                lake_width
            };
            let downhill_chunk = sim.get(downhill_pos).expect("How can this not work?");
            let coeffs =
                river_spline_coeffs(neighbor_pos, chunkj.river.spline_derivative, downhill_wpos);
            let (direction, coeffs, downhill_chunk, river_t, river_pos, river_dist) = match kind {
                RiverKind::River { .. } => {
                    if let Some((t, pt, dist)) = quadratic_nearest_point(&coeffs, wposf) {
                        (direction, coeffs, downhill_chunk, t, pt, dist.sqrt())
                    } else {
                        let ndist = wposf.distance_squared(neighbor_pos);
                        let ddist = wposf.distance_squared(downhill_wpos);
                        let (closest_pos, closest_dist, closest_t) = if ndist <= ddist {
                            (neighbor_pos, ndist, 0.0)
                        } else {
                            (downhill_wpos, ddist, 1.0)
                        };
                        (
                            direction,
                            coeffs,
                            downhill_chunk,
                            closest_t,
                            closest_pos,
                            closest_dist.sqrt(),
                        )
                    }
                },
                RiverKind::Lake { neighbor_pass_pos } => {
                    let pass_dist = neighbor_pass_pos
                        .map2(
                            neighbor_pos
                                .map2(TerrainChunkSize::RECT_SIZE, |f, g| (f as i32, g as i32)),
                            |e, (f, g)| ((e - f) / g).abs(),
                        )
                        .reduce_partial_max();
                    let spline_derivative = river.spline_derivative;
                    let neighbor_pass_pos = if pass_dist <= 1 {
                        neighbor_pass_pos
                    } else {
                        downhill_wpos.map(|e| e as i32)
                    };
                    let pass_dist = neighbor_pass_pos
                        .map2(
                            neighbor_pos
                                .map2(TerrainChunkSize::RECT_SIZE, |f, g| (f as i32, g as i32)),
                            |e, (f, g)| ((e - f) / g).abs(),
                        )
                        .reduce_partial_max();
                    if pass_dist > 1 {
                        return (posj, chunkj, river, None);
                    }
                    let neighbor_pass_wpos = neighbor_pass_pos.map(|e| e as f64);
                    let neighbor_pass_pos = neighbor_pass_pos
                        .map2(TerrainChunkSize::RECT_SIZE, |e, sz: u32| e / sz as i32);
                    let coeffs =
                        river_spline_coeffs(neighbor_pos, spline_derivative, neighbor_pass_wpos);
                    let direction = neighbor_pos - neighbor_pass_wpos;
                    if let Some((t, pt, dist)) = quadratic_nearest_point(&coeffs, wposf) {
                        (
                            direction,
                            coeffs,
                            sim.get(neighbor_pass_pos).expect("Must already work"),
                            t,
                            pt,
                            dist.sqrt(),
                        )
                    } else {
                        let ndist = wposf.distance_squared(neighbor_pos);
                        /* let ddist = wposf.distance_squared(neighbor_pass_wpos); */
                        let (closest_pos, closest_dist, closest_t) = /*if ndist <= ddist */ {
                                (neighbor_pos, ndist, 0.0)
                            } /* else {
                                (neighbor_pass_wpos, ddist, 1.0)
                            } */;
                        (
                            direction,
                            coeffs,
                            sim.get(neighbor_pass_pos).expect("Must already work"),
                            closest_t,
                            closest_pos,
                            closest_dist.sqrt(),
                        )
                    }
                },
                RiverKind::Ocean => {
                    let ndist = wposf.distance_squared(neighbor_pos);
                    let (closest_pos, closest_dist, closest_t) = (neighbor_pos, ndist, 0.0);
                    (
                        direction,
                        coeffs,
                        sim.get(closest_pos.map2(TerrainChunkSize::RECT_SIZE, |e, sz: u32| {
                            e as i32 / sz as i32
                        }))
                        .expect("Must already work"),
                        closest_t,
                        closest_pos,
                        closest_dist.sqrt(),
                    )
                },
            };
            let river_width_max =
                if let Some(RiverKind::River { cross_section }) = downhill_chunk.river.river_kind {
                    cross_section.x as f64
                } else {
                    lake_width
                };
            let river_width_noise = (sim.gen_ctx.small_nz.get((river_pos.div(16.0)).into_array()))
                .max(-1.0)
                .min(1.0)
                .mul(0.5)
                .sub(0.5) as f64;
            let river_width = Lerp::lerp(
                river_width_min,
                river_width_max,
                river_t.max(0.0).min(1.0).powf(0.5),
            );

            let river_width = river_width * (1.0 + river_width_noise * 0.3);
            // To find the distance, we just evaluate the quadratic equation at river_t and
            // see if it's within width (but we should be able to use it for a
            // lot more, and this probably isn't the very best approach anyway
            // since it will bleed out). let river_pos = coeffs.x * river_t *
            // river_t + coeffs.y * river_t + coeffs.z;
            let res = Vec2::new(0.0, (river_dist - (river_width * 0.5).max(1.0)).max(0.0));
            (
                posj,
                chunkj,
                river,
                Some((
                    direction,
                    res,
                    river_width,
                    (river_t, (river_pos, coeffs), downhill_chunk),
                )),
            )
        });

        // Find the average distance to each neighboring body of water.
        let mut river_count = 0.0f64;
        let mut overlap_count = 0.0f64;
        let mut river_distance_product = 1.0f64;
        let mut river_overlap_distance_product = 0.0f64;
        let mut max_river = None;
        let mut max_key = None;
        // IDEA:
        // For every "nearby" chunk, check whether it is a river.  If so, find the
        // closest point on the river segment to wposf (if two point are
        // equidistant, choose the earlier one), calling this point river_pos
        // and the length (from 0 to 1) along the river segment for the nearby
        // chunk river_t.  Let river_dist be the distance from river_pos to wposf.
        //
        // Let river_alt be the interpolated river height at this point
        // (from the alt/water altitude at the river, to the alt/water_altitude of the
        // downhill river, increasing with river_t).
        //
        // Now, if river_dist is <= river_width * 0.5, then we don't care what altitude
        // we use, and mark that we are on a river (we decide what river to use
        // using a heuristic, and set the solely according to the computed
        // river_alt for that point).
        //
        // Otherwise, we let dist = river_dist - river_width * 0.5.
        //
        // If dist >= TerrainChunkSize::RECT_SIZE.x, we don't include this river in the
        // calculation of the correct altitude for this point.
        //
        // Otherwise (i.e. dist < TerrainChunkSize::RECT_SIZE.x), we want to bias the
        // altitude of this point towards the altitude of the river.
        // Specifically, as the dist goes from TerrainChunkSize::RECT_SIZE.x to
        // 0, the weighted altitude of this point should go from
        // alt to river_alt.
        neighbor_river_data.for_each(|(river_chunk_idx, river_chunk, river, dist)| {
            match river.river_kind {
                Some(kind) => {
                    if kind.is_river() && !dist.is_some() {
                        // Ostensibly near a river segment, but not "usefully" so (there is no
                        // closest point between t = 0.0 and t = 1.0).
                        return;
                    } else {
                        let river_dist = dist.map(|(_, dist, _, (river_t, _, downhill_river))| {
                            let downhill_height = if kind.is_river() {
                                Lerp::lerp(
                                    river_chunk.alt.max(river_chunk.water_alt),
                                    downhill_river.alt.max(downhill_river.water_alt),
                                    river_t as f32,
                                ) as f64
                            } else {
                                let neighbor_pos =
                                    river_chunk_idx.map(|e| e as f64) * neighbor_coef;
                                if dist.y == 0.0 {
                                    -(wposf - neighbor_pos).magnitude()
                                } else {
                                    -(wposf - neighbor_pos).magnitude()
                                }
                            };
                            (Reverse((dist.x, dist.y)), downhill_height)
                        });
                        let river_dist = river_dist.or_else(|| {
                            if !kind.is_river() {
                                let neighbor_pos =
                                    river_chunk_idx.map(|e| e as f64) * neighbor_coef;
                                let dist = (wposf - neighbor_pos).magnitude();
                                let dist_upon =
                                    (dist - TerrainChunkSize::RECT_SIZE.x as f64 * 0.5).max(0.0);
                                let dist_ = if dist == 0.0 { f64::INFINITY } else { -dist };
                                Some((Reverse((0.0, dist_upon)), dist_))
                            } else {
                                None
                            }
                        });
                        let river_key = (river_dist, Reverse(kind));
                        if max_key < Some(river_key) {
                            max_river = Some((river_chunk_idx, river_chunk, river, dist));
                            max_key = Some(river_key);
                        }
                    }

                    // NOTE: we scale by the distance to the river divided by the difference
                    // between the edge of the river that we intersect, and the remaining distance
                    // until the nearest point in "this" chunk (i.e. the one whose top-left corner
                    // is chunk_pos) that is at least 2 chunks away from the river source.
                    if let Some((_, dist, _, (river_t, _, downhill_river_chunk))) = dist {
                        let max_distance = if !river.is_river() {
                            /*(*/
                            TerrainChunkSize::RECT_SIZE.x as f64 /* * (1.0 - (2.0f64.sqrt() / 2.0))) + 4.0*/ - lake_width * 0.5
                        } else {
                            TerrainChunkSize::RECT_SIZE.x as f64
                        };
                        let scale_factor = max_distance;
                        let river_dist = dist.y;

                        if !(dist.x == 0.0 && river_dist < scale_factor) {
                            return;
                        }
                        // We basically want to project outwards from river_pos, along the current
                        // tangent line, to chunks <= river_width * 1.0 away from this
                        // point.  We *don't* want to deal with closer chunks because they

                        // NOTE: river_width <= 2 * max terrain chunk size width, so this should not
                        // lead to division by zero.
                        // NOTE: If distance = 0.0 this goes to zero, which is desired since it
                        // means points that actually intersect with rivers will not be interpolated
                        // with the "normal" height of this point.
                        // NOTE: We keep the maximum at 1.0 so we don't undo work from another river
                        // just by being far away.
                        let river_scale = river_dist / scale_factor;
                        let river_alt =
                            Lerp::lerp(river_chunk.alt, downhill_river_chunk.alt, river_t as f32);
                        let river_alt = Lerp::lerp(river_alt, alt, river_scale as f32);
                        let river_alt_diff = river_alt - alt;
                        let river_alt_inv = river_alt_diff as f64;
                        river_overlap_distance_product += (1.0 - river_scale) * river_alt_inv;
                        overlap_count += 1.0 - river_scale;
                        river_count += 1.0;
                        river_distance_product *= river_scale;
                    }
                }
                None => {}
            }
        });

        let river_scale_factor = if river_count == 0.0 {
            1.0
        } else {
            let river_scale_factor = river_distance_product;
            if river_scale_factor == 0.0 {
                0.0
            } else {
                river_scale_factor.powf(if river_count == 0.0 {
                    1.0
                } else {
                    1.0 / river_count
                })
            }
        };

        let alt_for_river = alt
            + if overlap_count == 0.0 {
                0.0
            } else {
                river_overlap_distance_product / overlap_count
            } as f32;

        let riverless_alt_delta = (sim.gen_ctx.small_nz.get(
            (wposf_turb.div(200.0 * (32.0 / TerrainChunkSize::RECT_SIZE.x as f64))).into_array(),
        ) as f32)
            .min(1.0)
            .max(-1.0)
            .abs()
            .mul(3.0)
            + (sim.gen_ctx.small_nz.get(
                (wposf_turb.div(400.0 * (32.0 / TerrainChunkSize::RECT_SIZE.x as f64)))
                    .into_array(),
            ) as f32)
                .min(1.0)
                .max(-1.0)
                .abs()
                .mul(3.0);

        let downhill = sim_chunk.downhill;
        let downhill_pos = downhill.and_then(|downhill_pos| sim.get(downhill_pos));
        debug_assert!(sim_chunk.water_alt >= CONFIG.sea_level);

        let downhill_water_alt = downhill_pos
            .map(|downhill_chunk| {
                downhill_chunk
                    .water_alt
                    .min(sim_chunk.water_alt)
                    .max(sim_chunk.alt.min(sim_chunk.water_alt))
            })
            .unwrap_or(CONFIG.sea_level);

        let river_gouge = 0.5;
        let (_in_water, water_dist, alt_, water_level, riverless_alt, warp_factor) = if let Some(
            (max_border_river_pos, river_chunk, max_border_river, max_border_river_dist),
        ) =
            max_river
        {
            // This is flowing into a lake, or a lake, or is at least a non-ocean tile.
            //
            // If we are <= water_alt, we are in the lake; otherwise, we are flowing into
            // it.
            let (in_water, water_dist, new_alt, new_water_alt, riverless_alt, warp_factor) =
                max_border_river
                    .river_kind
                    .and_then(|river_kind| {
                        match river_kind {
                            RiverKind::River { cross_section } => {
                                if max_border_river_dist.map(|(_, dist, _, _)| dist)
                                    != Some(Vec2::zero())
                                {
                                    return None;
                                }
                                let (
                                    _,
                                    _,
                                    river_width,
                                    (river_t, (river_pos, _), downhill_river_chunk),
                                ) = max_border_river_dist.unwrap();
                                let river_alt = Lerp::lerp(
                                    river_chunk.alt.max(river_chunk.water_alt),
                                    downhill_river_chunk.alt.max(downhill_river_chunk.water_alt),
                                    river_t as f32,
                                );
                                let new_alt = river_alt - river_gouge;
                                let river_dist = wposf.distance(river_pos);
                                let river_height_factor = river_dist / (river_width * 0.5);

                                let valley_alt = Lerp::lerp(
                                    new_alt - cross_section.y.max(1.0),
                                    new_alt - 1.0,
                                    (river_height_factor * river_height_factor) as f32,
                                );

                                Some((
                                    true,
                                    Some((river_dist - river_width * 0.5) as f32),
                                    valley_alt,
                                    new_alt,
                                    alt, //river_alt + cross_section.y.max(1.0),
                                    0.0,
                                ))
                            },
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| {
                        max_border_river
                            .river_kind
                            .map(|river_kind| {
                                match river_kind {
                                    RiverKind::Ocean => {
                                        let (
                                            _,
                                            dist,
                                            river_width,
                                            (river_t, (river_pos, _), downhill_river_chunk),
                                        ) = if let Some(dist) = max_border_river_dist {
                                            dist
                                        } else {
                                            error!(
                                                ?max_border_river,
                                                ?chunk_pos,
                                                ?max_border_river_pos,
                                                "downhill error details"
                                            );
                                            panic!(
                                                "Oceans should definitely have a downhill! \
                                                 ...Right?"
                                            );
                                        };
                                        let lake_water_alt = Lerp::lerp(
                                            river_chunk.alt.max(river_chunk.water_alt),
                                            downhill_river_chunk
                                                .alt
                                                .max(downhill_river_chunk.water_alt),
                                            river_t as f32,
                                        );

                                        if dist == Vec2::zero() {
                                            let river_dist = wposf.distance(river_pos);
                                            let _river_height_factor =
                                                river_dist / (river_width * 0.5);
                                            return (
                                                true,
                                                Some((river_dist - river_width * 0.5) as f32),
                                                alt_for_river
                                                    .min(lake_water_alt - 1.0 - river_gouge),
                                                lake_water_alt - river_gouge,
                                                alt_for_river.max(lake_water_alt),
                                                0.0,
                                            );
                                        }

                                        (
                                            river_scale_factor <= 1.0,
                                            Some(
                                                (wposf.distance(river_pos) - river_width * 0.5)
                                                    as f32,
                                            ),
                                            alt_for_river,
                                            downhill_water_alt,
                                            alt_for_river,
                                            river_scale_factor as f32,
                                        )
                                    },
                                    RiverKind::Lake { .. } => {
                                        let lake_dist = (max_border_river_pos.map(|e| e as f64)
                                            * neighbor_coef)
                                            .distance(wposf);
                                        let downhill_river_chunk = max_border_river_pos;
                                        let lake_id_dist = downhill_river_chunk - chunk_pos;
                                        let in_bounds = lake_id_dist.x >= -1
                                            && lake_id_dist.y >= -1
                                            && lake_id_dist.x <= 1
                                            && lake_id_dist.y <= 1;
                                        let in_bounds = in_bounds
                                            && (lake_id_dist.x >= 0 && lake_id_dist.y >= 0);
                                        let (_, dist, _, (river_t, _, downhill_river_chunk)) =
                                            if let Some(dist) = max_border_river_dist {
                                                dist
                                            } else if lake_dist
                                                <= TerrainChunkSize::RECT_SIZE.x as f64 * 1.0
                                                || in_bounds
                                            {
                                                let gouge_factor = 0.0;
                                                return (
                                                    in_bounds
                                                        || downhill_water_alt
                                                            .max(river_chunk.water_alt)
                                                            > alt_for_river,
                                                    Some(lake_dist as f32),
                                                    alt_for_river,
                                                    (downhill_water_alt.max(river_chunk.water_alt)
                                                        - river_gouge),
                                                    alt_for_river,
                                                    river_scale_factor as f32
                                                        * (1.0 - gouge_factor),
                                                );
                                            } else {
                                                return (
                                                    false,
                                                    Some(lake_dist as f32),
                                                    alt_for_river,
                                                    downhill_water_alt,
                                                    alt_for_river,
                                                    river_scale_factor as f32,
                                                );
                                            };

                                        let lake_dist = dist.y;
                                        let lake_water_alt = Lerp::lerp(
                                            river_chunk.alt.max(river_chunk.water_alt),
                                            downhill_river_chunk
                                                .alt
                                                .max(downhill_river_chunk.water_alt),
                                            river_t as f32,
                                        );
                                        if dist == Vec2::zero() {
                                            return (
                                                true,
                                                Some(lake_dist as f32),
                                                alt_for_river
                                                    .min(lake_water_alt - 1.0 - river_gouge),
                                                lake_water_alt - river_gouge,
                                                alt_for_river.max(lake_water_alt),
                                                0.0,
                                            );
                                        }
                                        if lake_dist <= TerrainChunkSize::RECT_SIZE.x as f64 * 1.0
                                            || in_bounds
                                        {
                                            let gouge_factor = if in_bounds && lake_dist <= 1.0 {
                                                1.0
                                            } else {
                                                0.0
                                            };
                                            let in_bounds_ = lake_dist
                                                <= TerrainChunkSize::RECT_SIZE.x as f64 * 0.5;
                                            if gouge_factor == 1.0 {
                                                return (
                                                    true,
                                                    Some(lake_dist as f32),
                                                    alt.min(lake_water_alt - 1.0 - river_gouge),
                                                    downhill_water_alt.max(lake_water_alt)
                                                        - river_gouge,
                                                    alt.max(lake_water_alt),
                                                    0.0,
                                                );
                                            } else {
                                                return (
                                                    true,
                                                    Some(lake_dist as f32),
                                                    alt_for_river,
                                                    if in_bounds_ {
                                                        downhill_water_alt.max(lake_water_alt)
                                                    } else {
                                                        downhill_water_alt
                                                    } - river_gouge,
                                                    alt_for_river,
                                                    river_scale_factor as f32
                                                        * (1.0 - gouge_factor),
                                                );
                                            }
                                        }
                                        (
                                            river_scale_factor <= 1.0,
                                            Some(lake_dist as f32),
                                            alt_for_river,
                                            downhill_water_alt,
                                            alt_for_river,
                                            river_scale_factor as f32,
                                        )
                                    },
                                    RiverKind::River { .. } => {
                                        let (_, _, river_width, (_, (river_pos, _), _)) =
                                            max_border_river_dist.unwrap();
                                        let river_dist = wposf.distance(river_pos);

                                        // FIXME: Make water altitude accurate.
                                        (
                                            river_scale_factor <= 1.0,
                                            Some((river_dist - river_width * 0.5) as f32),
                                            alt_for_river,
                                            downhill_water_alt,
                                            alt, //alt_for_river,
                                            river_scale_factor as f32,
                                        )
                                    },
                                }
                            })
                            .unwrap_or((
                                false,
                                None,
                                alt_for_river,
                                downhill_water_alt,
                                alt, //alt_for_river,
                                river_scale_factor as f32,
                            ))
                    });
            (
                in_water,
                water_dist,
                new_alt,
                new_water_alt,
                riverless_alt,
                warp_factor,
            )
        } else {
            (
                false,
                None,
                alt_for_river,
                downhill_water_alt,
                alt, //alt_for_river,
                1.0,
            )
        };
        let warp_factor = warp_factor * chunk_warp_factor;
        // NOTE: To disable warp, uncomment this line.
        // let warp_factor = 0.0;

        let riverless_alt_delta = Lerp::lerp(0.0, riverless_alt_delta, warp_factor);
        let alt = alt_ + riverless_alt_delta;
        let riverless_alt = riverless_alt + riverless_alt_delta;
        let basement =
            alt + sim.get_interpolated_monotone(wpos, |chunk| chunk.basement.sub(chunk.alt))?;

        let rock = (sim.gen_ctx.small_nz.get(
            Vec3::new(wposf.x, wposf.y, alt as f64)
                .div(100.0)
                .into_array(),
        ) as f32)
            //.mul(water_dist.map(|wd| (wd / 2.0).clamped(0.0, 1.0).powf(0.5)).unwrap_or(1.0))
            .mul(rockiness)
            .sub(0.4)
            .max(0.0)
            .mul(8.0);

        // Columns near water have a more stable temperature and so get pushed towards
        // the average (0)
        let temp = Lerp::lerp(
            Lerp::lerp(temp, 0.0, 0.1),
            temp,
            water_dist
                .map(|water_dist| water_dist / 20.0)
                .unwrap_or(1.0)
                .clamped(0.0, 1.0),
        );
        // Columns near water get a humidity boost
        let humidity = Lerp::lerp(
            Lerp::lerp(humidity, 1.0, 0.25),
            humidity,
            water_dist
                .map(|water_dist| water_dist / 20.0)
                .unwrap_or(1.0)
                .clamped(0.0, 1.0),
        );

        let wposf3d = Vec3::new(wposf.x, wposf.y, alt as f64);

        let marble_small = (sim.gen_ctx.hill_nz.get((wposf3d.div(3.0)).into_array()) as f32)
            .powf(3.0)
            .add(1.0)
            .mul(0.5);
        let marble = (sim.gen_ctx.hill_nz.get((wposf3d.div(48.0)).into_array()) as f32)
            .mul(0.75)
            .add(1.0)
            .mul(0.5)
            .add(marble_small.sub(0.5).mul(0.25));

        // Colours
        let Colors {
            cold_grass,
            warm_grass,
            dark_grass,
            wet_grass,
            cold_stone,
            hot_stone,
            warm_stone,
            beach_sand,
            desert_sand,
            snow,
            snow_moss,
            stone_col,
            dirt_low,
            dirt_high,
            snow_high,
            warm_stone_high,
            grass_high,
            tropical_high,
        } = index.colors.column;

        let cold_grass = cold_grass.into();
        let warm_grass = warm_grass.into();
        let dark_grass = dark_grass.into();
        let wet_grass = wet_grass.into();
        let cold_stone = cold_stone.into();
        let hot_stone = hot_stone.into();
        let warm_stone: Rgb<f32> = warm_stone.into();
        let beach_sand = beach_sand.into();
        let desert_sand = desert_sand.into();
        let snow = snow.into();
        let stone_col = stone_col.into();
        let dirt_low: Rgb<f32> = dirt_low.into();
        let dirt_high = dirt_high.into();
        let snow_high = snow_high.into();
        let warm_stone_high = warm_stone_high.into();
        let grass_high = grass_high.into();
        let tropical_high = tropical_high.into();

        let dirt = Lerp::lerp(dirt_low, dirt_high, marble);
        let tundra = Lerp::lerp(snow, snow_high, 0.4 + marble * 0.6);
        let dead_tundra = Lerp::lerp(warm_stone, warm_stone_high, marble);
        let cliff = Rgb::lerp(cold_stone, hot_stone, marble);

        let grass = Rgb::lerp(
            cold_grass,
            warm_grass,
            marble.sub(0.5).add(1.0.sub(humidity).mul(0.5)).powf(1.5),
        );
        let snow_moss = Rgb::lerp(snow_moss.into(), cold_grass, 0.4 + marble.powf(1.5) * 0.6);
        let moss = Rgb::lerp(dark_grass, cold_grass, marble.powf(1.5));
        let rainforest = Rgb::lerp(wet_grass, warm_grass, marble.powf(1.5));
        let sand = Rgb::lerp(beach_sand, desert_sand, marble);

        let tropical = Rgb::lerp(
            Rgb::lerp(
                grass,
                grass_high,
                marble_small
                    .sub(0.5)
                    .mul(0.2)
                    .add(0.75.mul(1.0.sub(humidity)))
                    .powf(0.667),
            ),
            tropical_high,
            marble.powf(1.5).sub(0.5).mul(4.0),
        );

        // For below desert humidity, we are always sand or rock, depending on altitude
        // and temperature.
        let ground = Lerp::lerp(
            Lerp::lerp(
                dead_tundra,
                sand,
                temp.sub(CONFIG.snow_temp)
                    .div(CONFIG.desert_temp.sub(CONFIG.snow_temp))
                    .mul(0.5),
            ),
            dirt,
            humidity
                .sub(CONFIG.desert_hum)
                .div(CONFIG.forest_hum.sub(CONFIG.desert_hum))
                .mul(1.0),
        );

        let sub_surface_color = Lerp::lerp(cliff, ground, alt.sub(basement).mul(0.25));

        // From desert to forest humidity, we go from tundra to dirt to grass to moss to
        // sand, depending on temperature.
        let ground = Rgb::lerp(
            ground,
            Rgb::lerp(
                Rgb::lerp(
                    Rgb::lerp(
                        Rgb::lerp(
                            tundra,
                            // snow_temp to temperate_temp
                            dirt,
                            temp.sub(CONFIG.snow_temp)
                                .div(CONFIG.temperate_temp.sub(CONFIG.snow_temp))
                                /*.sub((marble - 0.5) * 0.05)
                                .mul(256.0)*/
                                .mul(1.0),
                        ),
                        // temperate_temp to tropical_temp
                        grass,
                        temp.sub(CONFIG.temperate_temp)
                            .div(CONFIG.tropical_temp.sub(CONFIG.temperate_temp))
                            .mul(4.0),
                    ),
                    // tropical_temp to desert_temp
                    moss,
                    temp.sub(CONFIG.tropical_temp)
                        .div(CONFIG.desert_temp.sub(CONFIG.tropical_temp))
                        .mul(1.0),
                ),
                // above desert_temp
                sand,
                temp.sub(CONFIG.desert_temp)
                    .div(1.0 - CONFIG.desert_temp)
                    .mul(4.0),
            ),
            humidity
                .sub(CONFIG.desert_hum)
                .div(CONFIG.forest_hum.sub(CONFIG.desert_hum))
                .mul(1.25),
        );
        // From forest to jungle humidity, we go from snow to dark grass to grass to
        // tropics to sand depending on temperature.
        let ground = Rgb::lerp(
            ground,
            Rgb::lerp(
                Rgb::lerp(
                    Rgb::lerp(
                        snow_moss,
                        // temperate_temp to tropical_temp
                        grass,
                        temp.sub(CONFIG.temperate_temp)
                            .div(CONFIG.tropical_temp.sub(CONFIG.temperate_temp))
                            .mul(4.0),
                    ),
                    // tropical_temp to desert_temp
                    tropical,
                    temp.sub(CONFIG.tropical_temp)
                        .div(CONFIG.desert_temp.sub(CONFIG.tropical_temp))
                        .mul(1.0),
                ),
                // above desert_temp
                sand,
                temp.sub(CONFIG.desert_temp)
                    .div(1.0 - CONFIG.desert_temp)
                    .mul(4.0),
            ),
            humidity
                .sub(CONFIG.forest_hum)
                .div(CONFIG.jungle_hum.sub(CONFIG.forest_hum))
                .mul(1.0),
        );
        // From jungle humidity upwards, we go from snow to grass to rainforest to
        // tropics to sand.
        let ground = Rgb::lerp(
            ground,
            Rgb::lerp(
                Rgb::lerp(
                    Rgb::lerp(
                        snow_moss,
                        // temperate_temp to tropical_temp
                        rainforest,
                        temp.sub(CONFIG.temperate_temp)
                            .div(CONFIG.tropical_temp.sub(CONFIG.temperate_temp))
                            .mul(4.0),
                    ),
                    // tropical_temp to desert_temp
                    tropical,
                    temp.sub(CONFIG.tropical_temp)
                        .div(CONFIG.desert_temp.sub(CONFIG.tropical_temp))
                        .mul(4.0),
                ),
                // above desert_temp
                sand,
                temp.sub(CONFIG.desert_temp)
                    .div(1.0 - CONFIG.desert_temp)
                    .mul(4.0),
            ),
            humidity.sub(CONFIG.jungle_hum).mul(1.0),
        );

        // Snow covering
        let snow_cover = temp
            .sub(CONFIG.snow_temp)
            .max(-humidity.sub(CONFIG.desert_hum))
            .mul(16.0)
            .add((marble_small - 0.5) * 0.5);
        let (alt, ground, sub_surface_color, snow_cover) = if snow_cover <= 0.5 && alt > water_level
        {
            // Allow snow cover.
            (
                alt + 1.0 - snow_cover.max(0.0),
                Rgb::lerp(snow, ground, snow_cover),
                Lerp::lerp(sub_surface_color, ground, alt.sub(basement).mul(0.15)),
                true,
            )
        } else {
            (alt, ground, sub_surface_color, false)
        };

        // Make river banks not have grass
        let ground = water_dist
            .map(|wd| Lerp::lerp(sub_surface_color, ground, (wd / 3.0).clamped(0.0, 1.0)))
            .unwrap_or(ground);

        let near_ocean = max_river.and_then(|(_, _, river_data, _)| {
            if (river_data.is_lake() || river_data.river_kind == Some(RiverKind::Ocean))
                && alt <= water_level.max(CONFIG.sea_level + 5.0)
            {
                Some(water_level)
            } else {
                None
            }
        });

        let ocean_level = if let Some(_sea_level) = near_ocean {
            alt - CONFIG.sea_level
        } else {
            5.0
        };

        let gradient = sim.get_gradient_approx(chunk_pos);

        let path = sim.get_nearest_path(wpos);
        let cave = sim.get_nearest_cave(wpos);

        Some(ColumnSample {
            alt,
            riverless_alt,
            basement,
            chaos,
            water_level,
            warp_factor,
            surface_color: Rgb::lerp(
                sub_surface_color,
                Rgb::lerp(
                    Rgb::lerp(cliff, sand, alt.sub(basement).mul(0.25)),
                    // Land
                    ground,
                    // Beach
                    ((ocean_level - 1.0) / 2.0).max(0.0),
                ),
                surface_veg,
            ),
            sub_surface_color,
            // No growing directly on bedrock.
            // And, no growing on sites that don't want them TODO: More precise than this when we
            // apply trees as a post-processing layer
            tree_density: if sim_chunk
                .sites
                .iter()
                .all(|site| index.sites[*site].spawn_rules(wpos).trees)
            {
                Lerp::lerp(0.0, tree_density, alt.sub(2.0).sub(basement).mul(0.5))
            } else {
                0.0
            },
            forest_kind: sim_chunk.forest_kind,
            marble,
            marble_small,
            rock,
            temp,
            humidity,
            spawn_rate,
            stone_col,
            water_dist,
            gradient,
            path,
            cave,
            snow_cover,

            chunk: sim_chunk,
        })
    }
}

#[derive(Clone)]
pub struct ColumnSample<'a> {
    pub alt: f32,
    pub riverless_alt: f32,
    pub basement: f32,
    pub chaos: f32,
    pub water_level: f32,
    pub warp_factor: f32,
    pub surface_color: Rgb<f32>,
    pub sub_surface_color: Rgb<f32>,
    pub tree_density: f32,
    pub forest_kind: ForestKind,
    pub marble: f32,
    pub marble_small: f32,
    pub rock: f32,
    pub temp: f32,
    pub humidity: f32,
    pub spawn_rate: f32,
    pub stone_col: Rgb<u8>,
    pub water_dist: Option<f32>,
    pub gradient: Option<f32>,
    pub path: Option<(f32, Vec2<f32>, Path, Vec2<f32>)>,
    pub cave: Option<(f32, Vec2<f32>, Cave, Vec2<f32>)>,
    pub snow_cover: bool,

    pub chunk: &'a SimChunk,
}
