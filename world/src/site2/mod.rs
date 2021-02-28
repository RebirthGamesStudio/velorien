mod gen;
mod plot;
mod tile;

use self::{
    plot::{Plot, PlotKind},
    tile::{TileGrid, Tile, TileKind, HazardKind, TILE_SIZE},
    gen::{Primitive, Fill, Structure},
};
use crate::{
    site::SpawnRules,
    util::{Grid, attempt, DHashSet, CARDINALS, SQUARE_4, SQUARE_9, LOCALITY},
    Canvas,
    Land,
};
use common::{
    terrain::{Block, BlockKind, SpriteKind, TerrainChunkSize},
    vol::RectVolSize,
    store::{Id, Store},
    astar::Astar,
    lottery::Lottery,
    spiral::Spiral2d,
};
use hashbrown::hash_map::DefaultHashBuilder;
use rand::prelude::*;
use vek::*;
use std::ops::Range;

#[derive(Default)]
pub struct Site {
    pub(crate) origin: Vec2<i32>,
    tiles: TileGrid,
    plots: Store<Plot>,
    plazas: Vec<Id<Plot>>,
    roads: Vec<Id<Plot>>,
}

impl Site {
    pub fn radius(&self) -> f32 {
        ((self.tiles.bounds.min.map(|e| e.abs()).reduce_max()
            .max(self.tiles.bounds.max.map(|e| e.abs()).reduce_max()) + 1) * tile::TILE_SIZE as i32) as f32
    }

    pub fn spawn_rules(&self, wpos: Vec2<i32>) -> SpawnRules {
        SpawnRules {
            trees: SQUARE_9
                .iter()
                .all(|&rpos| self.wpos_tile(wpos + rpos * tile::TILE_SIZE as i32).is_empty()),
            ..SpawnRules::default()
        }
    }

    pub fn bounds(&self) -> Aabr<i32> {
        let border = 1;
        Aabr {
            min: self.origin + self.tile_wpos(self.tiles.bounds.min - border),
            max: self.origin + self.tile_wpos(self.tiles.bounds.max + 1 + border),
        }
    }

    pub fn plot(&self, id: Id<Plot>) -> &Plot { &self.plots[id] }

    pub fn plots(&self) -> impl Iterator<Item = &Plot> + '_ { self.plots.values() }

    pub fn create_plot(&mut self, plot: Plot) -> Id<Plot> { self.plots.insert(plot) }

    pub fn blit_aabr(&mut self, aabr: Aabr<i32>, tile: Tile) {
        for y in 0..aabr.size().h {
            for x in 0..aabr.size().w {
                self.tiles.set(aabr.min + Vec2::new(x, y), tile.clone());
            }
        }
    }

    pub fn create_road(&mut self, land: &Land, rng: &mut impl Rng, a: Vec2<i32>, b: Vec2<i32>, w: i32) -> Option<Id<Plot>> {
        const MAX_ITERS: usize = 4096;
        let heuristic = |tile: &Vec2<i32>| {
            for y in 0..w {
                for x in 0..w {
                    if self.tiles.get(*tile + Vec2::new(x, y)).is_obstacle() {
                        return 1000.0;
                    }
                }
            }
            (tile.distance_squared(b) as f32).sqrt()
        };
        let path = Astar::new(MAX_ITERS, a, &heuristic, DefaultHashBuilder::default()).poll(
            MAX_ITERS,
            &heuristic,
            |tile| { let tile = *tile; CARDINALS.iter().map(move |dir| tile + *dir) },
            |a, b| {
                let alt_a = land.get_alt_approx(self.tile_center_wpos(*a));
                let alt_b = land.get_alt_approx(self.tile_center_wpos(*b));
                (alt_a - alt_b).abs() / TILE_SIZE as f32
            },
            |tile| *tile == b,
        ).into_path()?;

        let plot = self.create_plot(Plot {
            kind: PlotKind::Road(path.clone()),
            root_tile: a,
            tiles: path.clone().into_iter().collect(),
            seed: rng.gen(),
        });

        self.roads.push(plot);

        for &tile in path.iter() {
            for y in 0..w {
                for x in 0..w {
                    self.tiles.set(tile + Vec2::new(x, y), Tile {
                        kind: TileKind::Road,
                        plot: Some(plot),
                    });
                }
            }
        }

        Some(plot)
    }

    pub fn find_aabr(&mut self, search_pos: Vec2<i32>, area_range: Range<u32>, min_dims: Extent2<u32>) -> Option<(Aabr<i32>, Vec2<i32>)> {
        self.tiles.find_near(
            search_pos,
            |center, _| self.tiles.grow_aabr(center, area_range.clone(), min_dims)
                .ok()
                .filter(|aabr| {
                    (aabr.min.x..aabr.max.x).any(|x| self.tiles.get(Vec2::new(x, aabr.min.y - 1)).kind == TileKind::Road)
                    || (aabr.min.x..aabr.max.x).any(|x| self.tiles.get(Vec2::new(x, aabr.max.y)).kind == TileKind::Road)
                    || (aabr.min.y..aabr.max.y).any(|y| self.tiles.get(Vec2::new(aabr.min.x - 1, y)).kind == TileKind::Road)
                    || (aabr.min.y..aabr.max.y).any(|y| self.tiles.get(Vec2::new(aabr.max.x, y)).kind == TileKind::Road)
                }),
        )
    }

    pub fn find_roadside_aabr(&mut self, rng: &mut impl Rng, area_range: Range<u32>, min_dims: Extent2<u32>) -> Option<(Aabr<i32>, Vec2<i32>)> {
        let dir = Vec2::<f32>::zero().map(|_| rng.gen_range(-1.0..1.0)).normalized();
        let search_pos = if rng.gen() {
            self.plot(*self.plazas.choose(rng)?).root_tile + (dir * 4.0).map(|e: f32| e.round() as i32)
        } else {
            if let PlotKind::Road(path) = &self.plot(*self.roads.choose(rng)?).kind {
                *path.nodes().choose(rng)? + (dir * 1.0).map(|e: f32| e.round() as i32)
            } else {
                unreachable!()
            }
        };

        self.find_aabr(search_pos, area_range, min_dims)
    }

    pub fn make_plaza(&mut self, land: &Land, rng: &mut impl Rng) -> Id<Plot> {
        let pos = attempt(32, || {
            self.plazas
                .choose(rng)
                .map(|&p| self.plot(p).root_tile + (Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalized() * 24.0).map(|e| e as i32))
                .filter(|tile| !self.tiles.get(*tile).is_obstacle())
                .filter(|&tile| self
                    .plazas
                    .iter()
                    .all(|&p| self.plot(p).root_tile.distance_squared(tile) > 20i32.pow(2))
                    && rng.gen_range(0..48) > tile.map(|e| e.abs()).reduce_max())
        })
            .unwrap_or_else(Vec2::zero);

        let aabr = Aabr { min: pos + Vec2::broadcast(-3), max: pos + Vec2::broadcast(4) };
        let plaza = self.create_plot(Plot {
            kind: PlotKind::Plaza,
            root_tile: pos,
            tiles: aabr_tiles(aabr).collect(),
            seed: rng.gen(),
        });
        self.plazas.push(plaza);
        self.blit_aabr(aabr, Tile {
            kind: TileKind::Road,
            plot: Some(plaza),
        });

        let mut already_pathed = vec![plaza];
        // One major, one minor road
        for width in (1..=2).rev() {
            if let Some(&p) = self.plazas
                .iter()
                .filter(|p| !already_pathed.contains(p))
                .min_by_key(|&&p| self.plot(p).root_tile.distance_squared(pos))
            {
                self.create_road(land, rng, self.plot(p).root_tile, pos, width);
                already_pathed.push(p);
            } else {
                break;
            }
        }

        plaza
    }

    pub fn demarcate_obstacles(&mut self, land: &Land) {
        const SEARCH_RADIUS: u32 = 96;

        Spiral2d::new()
            .take((SEARCH_RADIUS * 2 + 1).pow(2) as usize)
            .for_each(|tile| {
                if let Some(kind) = wpos_is_hazard(land, self.tile_wpos(tile)) {
                    for &rpos in &SQUARE_4 {
                        // `get_mut` doesn't increase generation bounds
                        self.tiles.get_mut(tile - rpos - 1).map(|tile| tile.kind = TileKind::Hazard(kind));
                    }
                }
            });
    }

    pub fn generate(land: &Land, rng: &mut impl Rng, origin: Vec2<i32>) -> Self {
        let mut site = Site {
            origin,
            ..Site::default()
        };

        site.demarcate_obstacles(land);

        site.make_plaza(land, rng);

        let build_chance = Lottery::from(vec![
            (1.0, 0),
            (48.0, 1),
            (5.0, 2),
            (20.0, 3),
            (1.0, 4),
        ]);

        let mut castles = 0;

        for _ in 0..1000 {
            if site.plots.len() - site.plazas.len() > 80 {
                break;
            }

            match *build_chance.choose_seeded(rng.gen()) {
                // Plaza
                0 => {
                    site.make_plaza(land, rng);
                },
                // House
                1 => {
                    let size = (2.0 + rng.gen::<f32>().powf(8.0) * 3.0).round() as u32;
                    if let Some((aabr, _)) = attempt(10, || site.find_roadside_aabr(rng, 4..(size + 1).pow(2), Extent2::broadcast(size))) {
                        let plot = site.create_plot(Plot {
                            kind: PlotKind::House(plot::House::generate(land, rng, &site, aabr)),
                            root_tile: aabr.center(),
                            tiles: aabr_tiles(aabr).collect(),
                            seed: rng.gen(),
                        });

                        site.blit_aabr(aabr, Tile {
                            kind: TileKind::Building { levels: size - 1 + rng.gen_range(0..2) },
                            plot: Some(plot),
                        });
                    }
                },
                // Guard tower
                2 => {
                    if let Some((aabr, _)) = attempt(10, || site.find_roadside_aabr(rng, 4..4, Extent2::new(2, 2))) {
                        let plot = site.create_plot(Plot {
                            kind: PlotKind::Castle,
                            root_tile: aabr.center(),
                            tiles: aabr_tiles(aabr).collect(),
                            seed: rng.gen(),
                        });

                        site.blit_aabr(aabr, Tile {
                            kind: TileKind::Castle,
                            plot: Some(plot),
                        });
                    }
                },
                // Field
                3 => {
                    attempt(10, || {
                        let search_pos = attempt(16, || {
                            let tile = (Vec2::new(
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                            ).normalized() * rng.gen_range(32.0..48.0)).map(|e| e as i32);

                            if site
                                .plazas
                                .iter()
                                .all(|&p| site.plot(p).root_tile.distance_squared(tile) > 20i32.pow(2))
                                && rng.gen_range(0..48) > tile.map(|e| e.abs()).reduce_max()
                            {
                                Some(tile)
                            } else {
                                None
                            }
                        })
                            .unwrap_or_else(Vec2::zero);
                        site.tiles.find_near(
                            search_pos,
                            |center, _| site.tiles.grow_aabr(center, 9..25, Extent2::new(3, 3)).ok())
                    })
                    .map(|(aabr, _)| {
                        site.blit_aabr(aabr, Tile {
                            kind: TileKind::Field,
                            plot: None,
                        });
                    });
                },
                // Castle
                _ if castles < 1 => {
                    if let Some((aabr, _)) = attempt(10, || site.find_roadside_aabr(rng, 16 * 16..18 * 18, Extent2::new(16, 16))) {
                        let plot = site.create_plot(Plot {
                            kind: PlotKind::Castle,
                            root_tile: aabr.center(),
                            tiles: aabr_tiles(aabr).collect(),
                            seed: rng.gen(),
                        });

                        // Walls
                        site.blit_aabr(aabr, Tile {
                            kind: TileKind::Wall,
                            plot: Some(plot),
                        });

                        let tower = Tile {
                            kind: TileKind::Castle,
                            plot: Some(plot),
                        };
                        site.tiles.set(Vec2::new(aabr.min.x, aabr.min.y), tower.clone());
                        site.tiles.set(Vec2::new(aabr.max.x - 1, aabr.min.y), tower.clone());
                        site.tiles.set(Vec2::new(aabr.min.x, aabr.max.y - 1), tower.clone());
                        site.tiles.set(Vec2::new(aabr.max.x - 1, aabr.max.y - 1), tower.clone());

                        // Courtyard
                        site.blit_aabr(Aabr { min: aabr.min + 1, max: aabr.max - 1 } , Tile {
                            kind: TileKind::Road,
                            plot: Some(plot),
                        });

                        // Keep
                        site.blit_aabr(Aabr { min: aabr.center() - 3, max: aabr.center() + 3 }, Tile {
                            kind: TileKind::Castle,
                            plot: Some(plot),
                        });

                        castles += 1;
                    }
                },
                _ => {},
            }
        }

        site
    }

    pub fn wpos_tile_pos(&self, wpos2d: Vec2<i32>) -> Vec2<i32> {
        (wpos2d - self.origin).map(|e| e.div_euclid(TILE_SIZE as i32))
    }

    pub fn wpos_tile(&self, wpos2d: Vec2<i32>) -> &Tile {
        self.tiles.get(self.wpos_tile_pos(wpos2d))
    }

    pub fn tile_wpos(&self, tile: Vec2<i32>) -> Vec2<i32> {
        self.origin + tile * tile::TILE_SIZE as i32
    }

    pub fn tile_center_wpos(&self, tile: Vec2<i32>) -> Vec2<i32> {
        self.origin + tile * tile::TILE_SIZE as i32 + tile::TILE_SIZE as i32 / 2
    }

    pub fn render_tile(&self, canvas: &mut Canvas, dynamic_rng: &mut impl Rng, tpos: Vec2<i32>) {
        let tile = self.tiles.get(tpos);
        let twpos = self.tile_center_wpos(tpos);
        let cols = (-(TILE_SIZE as i32)..TILE_SIZE as i32 * 2).map(|y| (-(TILE_SIZE as i32)..TILE_SIZE as i32 * 2).map(move |x| (twpos + Vec2::new(x, y), Vec2::new(x, y)))).flatten();

        match &tile.kind {
            TileKind::Empty | TileKind::Hazard(_) => {},
            TileKind::Road => {
                let near_roads = CARDINALS
                    .map(|rpos| if self.tiles.get(tpos + rpos) == tile {
                        Some(LineSegment2 {
                            start: self.tile_center_wpos(tpos).map(|e| e as f32),
                            end: self.tile_center_wpos(tpos + rpos).map(|e| e as f32),
                        })
                    } else {
                        None
                    });

                cols.for_each(|(wpos2d, offs)| {
                    let wpos2df = wpos2d.map(|e| e as f32);
                    let nearest_road = near_roads
                        .iter()
                        .copied()
                        .filter_map(|line| Some(line?.projected_point(wpos2df)))
                        .min_by_key(|p| p.distance_squared(wpos2df) as i32);

                    let is_near_road = nearest_road.map_or(false, |r| r.distance_squared(wpos2df) < 3.0f32.powi(2));

                    if let Some(nearest_road) = nearest_road
                        .filter(|r| r.distance_squared(wpos2df) < 4.0f32.powi(2))
                    {
                        let road_alt = canvas.col(nearest_road.map(|e| e.floor() as i32)).map_or(0, |col| col.alt as i32);
                        (-4..5).for_each(|z| canvas.map(
                            Vec3::new(wpos2d.x, wpos2d.y, road_alt + z),
                            |b| if z > 0 {
                                Block::air(SpriteKind::Empty)
                            } else {
                                Block::new(BlockKind::Rock, Rgb::new(55, 45, 65))
                            },
                        ));
                    }
                });
            },
            _ => {},
        }
    }

    pub fn render(&self, canvas: &mut Canvas, dynamic_rng: &mut impl Rng) {
        let tile_aabr = Aabr {
            min: self.wpos_tile_pos(canvas.wpos()) - 1,
            max: self.wpos_tile_pos(canvas.wpos() + TerrainChunkSize::RECT_SIZE.map(|e| e as i32) + 2) + 3, // Round up, uninclusive, border
        };

        // Don't double-generate the same plot per chunk!
        let mut plots = DHashSet::default();

        for y in tile_aabr.min.y..tile_aabr.max.y {
            for x in tile_aabr.min.x..tile_aabr.max.x {
                self.render_tile(canvas, dynamic_rng, Vec2::new(x, y));

                if let Some(plot) = self.tiles.get(Vec2::new(x, y)).plot {
                    plots.insert(plot);
                }
            }
        }

        let mut plots_to_render = plots.into_iter().collect::<Vec<_>>();
        plots_to_render.sort_unstable();

        for plot in plots_to_render {
            let (prim_tree, fills) = match &self.plots[plot].kind {
                PlotKind::House(house) => house.render_collect(),
                _ => continue,
            };

            for fill in fills {
                let aabb = fill.get_bounds(&prim_tree);

                for x in aabb.min.x..aabb.max.x {
                    for y in aabb.min.y..aabb.max.y {
                        for z in aabb.min.z..aabb.max.z {
                            let pos = Vec3::new(x, y, z);

                            if let Some(block) = fill.sample_at(&prim_tree, pos) {
                                canvas.set(pos, block);
                            }
                        }
                    }
                }
            }
        }

        // canvas.foreach_col(|canvas, wpos2d, col| {
        //     let tile = self.wpos_tile(wpos2d);
        //     let seed = tile.plot.map_or(0, |p| self.plot(p).seed);
        //     match tile.kind {
        //         TileKind::Field /*| TileKind::Road*/ => (-4..5).for_each(|z| canvas.map(
        //             Vec3::new(wpos2d.x, wpos2d.y, col.alt as i32 + z),
        //             |b| if [
        //                 BlockKind::Grass,
        //                 BlockKind::Earth,
        //                 BlockKind::Sand,
        //                 BlockKind::Snow,
        //                 BlockKind::Rock,
        //             ]
        //             .contains(&b.kind()) {
        //                 match tile.kind {
        //                     TileKind::Field => Block::new(BlockKind::Earth, Rgb::new(40, 5 + (seed % 32) as u8, 0)),
        //                     TileKind::Road => Block::new(BlockKind::Rock, Rgb::new(55, 45, 65)),
        //                     _ => unreachable!(),
        //                 }
        //             } else {
        //                 b.with_sprite(SpriteKind::Empty)
        //             },
        //         )),
        //         TileKind::Building { levels } => {
        //             let base_alt = tile.plot.map(|p| self.plot(p)).map_or(col.alt as i32, |p| p.base_alt);
        //             for z in base_alt - 12..base_alt + 4 + 6 * levels as i32 {
        //                 canvas.set(
        //                     Vec3::new(wpos2d.x, wpos2d.y, z),
        //                     Block::new(BlockKind::Wood, Rgb::new(180, 90 + (seed % 64) as u8, 120))
        //                 );
        //             }
        //         },
        //         TileKind::Castle | TileKind::Wall => {
        //             let base_alt = tile.plot.map(|p| self.plot(p)).map_or(col.alt as i32, |p| p.base_alt);
        //             for z in base_alt - 12..base_alt + if tile.kind == TileKind::Wall { 24 } else { 40 } {
        //                 canvas.set(
        //                     Vec3::new(wpos2d.x, wpos2d.y, z),
        //                     Block::new(BlockKind::Wood, Rgb::new(40, 40, 55))
        //                 );
        //             }
        //         },
        //         _ => {},
        //     }
        // });
    }
}

pub fn test_site() -> Site { Site::generate(&Land::empty(), &mut thread_rng(), Vec2::zero()) }

fn wpos_is_hazard(land: &Land, wpos: Vec2<i32>) -> Option<HazardKind> {
    if land
        .get_chunk_at(wpos)
        .map_or(true, |c| c.river.near_water())
    {
        Some(HazardKind::Water)
    } else if let Some(gradient) = Some(land.get_gradient_approx(wpos)).filter(|g| *g > 0.8) {
        Some(HazardKind::Hill { gradient })
    } else {
        None
    }
}

pub fn aabr_tiles(aabr: Aabr<i32>) -> impl Iterator<Item=Vec2<i32>> {
    (0..aabr.size().h)
        .map(move |y| (0..aabr.size().w)
            .map(move |x| aabr.min + Vec2::new(x, y)))
        .flatten()
}

pub struct Plaza {}
