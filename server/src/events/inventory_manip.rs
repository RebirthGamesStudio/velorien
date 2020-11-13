use crate::{client::Client, Server, StateExt};
use common::{
    comp::{
        self, item::{self, tool::AbilityMap},
        slot::{self, Slot},
        Pos,
    },
    consts::MAX_PICKUP_RANGE,
    msg::ServerGeneral,
    recipe::default_recipe_book,
    sync::{Uid, WorldSyncExt},
    vol::ReadVol,
};
use comp::LightEmitter;
use rand::Rng;
use specs::{join::Join, world::WorldExt, Builder, Entity as EcsEntity, WriteStorage};
use tracing::{debug, error};
use vek::{Rgb, Vec3};

pub fn swap_lantern(
    storage: &mut WriteStorage<comp::LightEmitter>,
    entity: EcsEntity,
    lantern: &item::Lantern,
) {
    if let Some(light) = storage.get_mut(entity) {
        light.strength = lantern.strength();
        light.col = lantern.color();
    }
}

pub fn snuff_lantern(storage: &mut WriteStorage<comp::LightEmitter>, entity: EcsEntity) {
    storage.remove(entity);
}

#[allow(clippy::blocks_in_if_conditions)]
#[allow(clippy::same_item_push)] // TODO: Pending review in #587
pub fn handle_inventory(server: &mut Server, entity: EcsEntity, manip: comp::InventoryManip) {
    let state = server.state_mut();
    let mut dropped_items = Vec::new();
    let mut thrown_items = Vec::new();

    match manip {
        comp::InventoryManip::Pickup(uid) => {
            let picked_up_item: Option<comp::Item>;
            let item_entity = if let (Some((item, item_entity)), Some(inv)) = (
                state
                    .ecs()
                    .entity_from_uid(uid.into())
                    .and_then(|item_entity| {
                        state
                            .ecs()
                            .write_storage::<comp::Item>()
                            .get_mut(item_entity)
                            .map(|item| (item.clone(), item_entity))
                    }),
                state
                    .ecs()
                    .write_storage::<comp::Inventory>()
                    .get_mut(entity),
            ) {
                picked_up_item = Some(item.clone());
                if !within_pickup_range(
                    state.ecs().read_storage::<comp::Pos>().get(entity),
                    state.ecs().read_storage::<comp::Pos>().get(item_entity),
                ) {
                    debug!("Failed to pick up item as not within range, Uid: {}", uid);
                    return;
                };

                // Grab the health from the player and check if the player is dead.
                let healths = state.ecs().read_storage::<comp::Health>();
                if let Some(entity_health) = healths.get(entity) {
                    if entity_health.is_dead {
                        debug!("Failed to pick up item as the player is dead");
                        return; // If dead, don't continue
                    }
                }

                // Attempt to add the item to the player's inventory
                match inv.push(item) {
                    None => Some(item_entity),
                    Some(_) => None, // Inventory was full
                }
            } else {
                // Item entity/component could not be found - most likely because the player
                // attempted to pick up the same item very quickly before its deletion of the
                // world from the first pickup attempt was processed.
                debug!("Failed to get entity/component for item Uid: {}", uid);
                return;
            };

            let event = if let Some(item_entity) = item_entity {
                if let Err(err) = state.delete_entity_recorded(item_entity) {
                    // If this occurs it means the item was duped as it's been pushed to the
                    // player's inventory but also left on the ground
                    panic!("Failed to delete picked up item entity: {:?}", err);
                }
                comp::InventoryUpdate::new(comp::InventoryUpdateEvent::Collected(
                    picked_up_item.unwrap(),
                ))
            } else {
                comp::InventoryUpdate::new(comp::InventoryUpdateEvent::CollectFailed)
            };

            state.write_component(entity, event);
        },

        comp::InventoryManip::Collect(pos) => {
            let block = state.terrain().get(pos).ok().copied();

            if let Some(block) = block {
                if block.is_collectible() && state.can_set_block(pos) {
                    // Check if the block is within pickup range
                    if !within_pickup_range(
                        state.ecs().read_storage::<comp::Pos>().get(entity),
                        // We convert the Vec<i32> pos into a Vec<f32>, adding 0.5 to get the
                        // center of the block
                        Some(&Pos(pos.map(|e| e as f32 + 0.5))),
                    ) {
                        return;
                    };

                    if let Some(item) = comp::Item::try_reclaim_from_block(block) {
                        let (event, item_was_added) = if let Some(inv) = state
                            .ecs()
                            .write_storage::<comp::Inventory>()
                            .get_mut(entity)
                        {
                            match inv.push(item.clone()) {
                                None => (
                                    Some(comp::InventoryUpdate::new(
                                        comp::InventoryUpdateEvent::Collected(item),
                                    )),
                                    true,
                                ),
                                Some(_) => (
                                    Some(comp::InventoryUpdate::new(
                                        comp::InventoryUpdateEvent::CollectFailed,
                                    )),
                                    false,
                                ),
                            }
                        } else {
                            debug!(
                                "Can't add item to inventory: entity has no inventory ({:?})",
                                entity
                            );
                            (None, false)
                        };
                        if let Some(event) = event {
                            state.write_component(entity, event);
                            if item_was_added {
                                // we made sure earlier the block was not already modified this tick
                                state.set_block(pos, block.into_vacant())
                            };
                        }
                    } else {
                        debug!(
                            "Failed to reclaim item from block at pos={} or entity had no \
                             inventory",
                            pos
                        )
                    }
                } else {
                    debug!(
                        "Can't reclaim item from block at pos={}: block is not collectable or was \
                         already set this tick.",
                        pos
                    );
                }
            }
        },

        comp::InventoryManip::Use(slot) => {
            let mut inventories = state.ecs().write_storage::<comp::Inventory>();
            let inventory = if let Some(inventory) = inventories.get_mut(entity) {
                inventory
            } else {
                error!(
                    ?entity,
                    "Can't manipulate inventory, entity doesn't have one"
                );
                return;
            };

            let mut maybe_effect = None;

            let event = match slot {
                Slot::Inventory(slot) => {
                    use item::ItemKind;
                    let (is_equippable, lantern_opt) =
                        inventory
                            .get(slot)
                            .map_or((false, None), |i| match i.kind() {
                                ItemKind::Tool(_)
                                | ItemKind::Armor { .. }
                                | ItemKind::Glider(_) => (true, None),
                                ItemKind::Lantern(lantern) => (true, Some(lantern)),
                                _ => (false, None),
                            });
                    if is_equippable {
                        if let Some(loadout) = state.ecs().write_storage().get_mut(entity) {
                            if let Some(lantern) = lantern_opt {
                                swap_lantern(&mut state.ecs().write_storage(), entity, &lantern);
                            }
                            let map = state.ecs().fetch::<AbilityMap>();
                            slot::equip(slot, inventory, loadout, &map);
                            Some(comp::InventoryUpdateEvent::Used)
                        } else {
                            None
                        }
                    } else if let Some(item) = inventory.take(slot) {
                        match item.kind() {
                            ItemKind::Consumable { kind, effect, .. } => {
                                maybe_effect = Some(effect.clone());
                                Some(comp::InventoryUpdateEvent::Consumed(kind.clone()))
                            },
                            ItemKind::Throwable { kind, .. } => {
                                if let Some(pos) =
                                    state.ecs().read_storage::<comp::Pos>().get(entity)
                                {
                                    thrown_items.push((
                                        *pos,
                                        state
                                            .read_component_copied::<comp::Vel>(entity)
                                            .unwrap_or_default(),
                                        state
                                            .read_component_copied::<comp::Ori>(entity)
                                            .unwrap_or_default(),
                                        *kind,
                                    ));
                                }
                                Some(comp::InventoryUpdateEvent::Used)
                            },
                            ItemKind::Utility {
                                kind: comp::item::Utility::Collar,
                                ..
                            } => {
                                let reinsert = if let Some(pos) =
                                    state.read_storage::<comp::Pos>().get(entity)
                                {
                                    let uid = state
                                        .read_component_copied(entity)
                                        .expect("Expected player to have a UID");
                                    if (
                                        &state.read_storage::<comp::Alignment>(),
                                        &state.read_storage::<comp::Agent>(),
                                    )
                                        .join()
                                        .filter(|(alignment, _)| {
                                            alignment == &&comp::Alignment::Owned(uid)
                                        })
                                        .count()
                                        >= 3
                                    {
                                        true
                                    } else if let Some(tameable_entity) = {
                                        let nearest_tameable = (
                                            &state.ecs().entities(),
                                            &state.ecs().read_storage::<comp::Pos>(),
                                            &state.ecs().read_storage::<comp::Alignment>(),
                                        )
                                            .join()
                                            .filter(|(_, wild_pos, _)| {
                                                wild_pos.0.distance_squared(pos.0)
                                                    < 5.0f32.powf(2.0)
                                            })
                                            .filter(|(_, _, alignment)| {
                                                alignment == &&comp::Alignment::Wild
                                            })
                                            .min_by_key(|(_, wild_pos, _)| {
                                                (wild_pos.0.distance_squared(pos.0) * 100.0) as i32
                                            })
                                            .map(|(entity, _, _)| entity);
                                        nearest_tameable
                                    } {
                                        let _ = state
                                            .ecs()
                                            .write_storage()
                                            .insert(tameable_entity, comp::Alignment::Owned(uid));

                                        // Add to group system
                                        let clients = state.ecs().read_storage::<Client>();
                                        let uids = state.ecs().read_storage::<Uid>();
                                        let mut group_manager = state
                                            .ecs()
                                            .write_resource::<comp::group::GroupManager>(
                                        );
                                        group_manager.new_pet(
                                            tameable_entity,
                                            entity,
                                            &mut state.ecs().write_storage(),
                                            &state.ecs().entities(),
                                            &state.ecs().read_storage(),
                                            &uids,
                                            &mut |entity, group_change| {
                                                clients
                                                    .get(entity)
                                                    .and_then(|c| {
                                                        group_change
                                                            .try_map(|e| uids.get(e).copied())
                                                            .map(|g| (g, c))
                                                    })
                                                    .map(|(g, c)| {
                                                        c.send(ServerGeneral::GroupUpdate(g))
                                                    });
                                            },
                                        );

                                        let _ = state
                                            .ecs()
                                            .write_storage()
                                            .insert(tameable_entity, comp::Agent::default());
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                };

                                if reinsert {
                                    let _ = inventory.insert_or_stack(slot, item);
                                }

                                Some(comp::InventoryUpdateEvent::Used)
                            },
                            _ => {
                                inventory.insert_or_stack(slot, item).unwrap();
                                None
                            },
                        }
                    } else {
                        None
                    }
                },
                Slot::Equip(slot) => {
                    if let Some(loadout) = state.ecs().write_storage().get_mut(entity) {
                        if slot == slot::EquipSlot::Lantern {
                            snuff_lantern(&mut state.ecs().write_storage(), entity);
                        }
                        let map = state.ecs().fetch::<AbilityMap>();
                        slot::unequip(slot, inventory, loadout, &map);
                        Some(comp::InventoryUpdateEvent::Used)
                    } else {
                        error!(?entity, "Entity doesn't have a loadout, can't unequip...");
                        None
                    }
                },
            };

            drop(inventories);
            if let Some(effects) = maybe_effect {
                for effect in effects {
                    state.apply_effect(entity, effect, None);
                }
            }
            if let Some(event) = event {
                state.write_component(entity, comp::InventoryUpdate::new(event));
            }
        },

        comp::InventoryManip::Swap(a, b) => {
            let ecs = state.ecs();
            let mut inventories = ecs.write_storage();
            let mut loadouts = ecs.write_storage();
            let inventory = inventories.get_mut(entity);
            let loadout = loadouts.get_mut(entity);
            let map = state.ecs().fetch::<AbilityMap>();

            slot::swap(a, b, inventory, loadout, &map);

            // :/
            drop(loadouts);
            drop(inventories);
            drop(map);

            state.write_component(
                entity,
                comp::InventoryUpdate::new(comp::InventoryUpdateEvent::Swapped),
            );
        },

        comp::InventoryManip::Drop(slot) => {
            let map = state.ecs().fetch::<AbilityMap>();
            let item = match slot {
                Slot::Inventory(slot) => state
                    .ecs()
                    .write_storage::<comp::Inventory>()
                    .get_mut(entity)
                    .and_then(|inv| inv.remove(slot)),
                Slot::Equip(slot) => state
                    .ecs()
                    .write_storage()
                    .get_mut(entity)
                    .and_then(|ldt| slot::loadout_remove(slot, ldt, &map)),
            };
            drop(map);

            // FIXME: We should really require the drop and write to be atomic!
            if let (Some(mut item), Some(pos)) =
                (item, state.ecs().read_storage::<comp::Pos>().get(entity))
            {
                item.put_in_world();
                dropped_items.push((
                    *pos,
                    state
                        .read_component_copied::<comp::Ori>(entity)
                        .unwrap_or_default(),
                    item,
                ));
            }
            state.write_component(
                entity,
                comp::InventoryUpdate::new(comp::InventoryUpdateEvent::Dropped),
            );
        },

        comp::InventoryManip::CraftRecipe(recipe) => {
            if let Some(inv) = state
                .ecs()
                .write_storage::<comp::Inventory>()
                .get_mut(entity)
            {
                let recipe_book = default_recipe_book();
                let craft_result = recipe_book.get(&recipe).and_then(|r| r.perform(inv).ok());

                // FIXME: We should really require the drop and write to be atomic!
                if craft_result.is_some() {
                    let _ = state.ecs().write_storage().insert(
                        entity,
                        comp::InventoryUpdate::new(comp::InventoryUpdateEvent::Craft),
                    );
                }

                // Drop the item if there wasn't enough space
                if let Some(Some((item, amount))) = craft_result {
                    for _ in 0..amount {
                        dropped_items.push((
                            state
                                .read_component_copied::<comp::Pos>(entity)
                                .unwrap_or_default(),
                            state
                                .read_component_copied::<comp::Ori>(entity)
                                .unwrap_or_default(),
                            item.clone(),
                        ));
                    }
                }
            }
        },
    }

    // Drop items
    for (pos, ori, item) in dropped_items {
        let vel = *ori.0 * 5.0
            + Vec3::unit_z() * 10.0
            + Vec3::<f32>::zero().map(|_| rand::thread_rng().gen::<f32>() - 0.5) * 4.0;

        state
            .create_object(Default::default(), comp::object::Body::Pouch)
            .with(comp::Pos(pos.0 + Vec3::unit_z() * 0.25))
            .with(item)
            .with(comp::Vel(vel))
            .build();
    }

    let mut rng = rand::thread_rng();

    // Throw items
    for (pos, vel, ori, kind) in thrown_items {
        let vel = match kind {
            item::Throwable::Firework(_) => Vec3::new(
                rng.gen_range(-15.0, 15.0),
                rng.gen_range(-15.0, 15.0),
                rng.gen_range(80.0, 110.0),
            ),
            _ => {
                vel.0
                    + *ori.0 * 20.0
                    + Vec3::unit_z() * 15.0
                    + Vec3::<f32>::zero().map(|_| rand::thread_rng().gen::<f32>() - 0.5) * 4.0
            },
        };

        let uid = state.read_component_copied::<Uid>(entity);

        let mut new_entity = state
            .create_object(Default::default(), match kind {
                item::Throwable::Bomb => comp::object::Body::Bomb,
                item::Throwable::Firework(reagent) => match reagent {
                    item::Reagent::Blue => comp::object::Body::FireworkBlue,
                    item::Reagent::Green => comp::object::Body::FireworkGreen,
                    item::Reagent::Purple => comp::object::Body::FireworkPurple,
                    item::Reagent::Red => comp::object::Body::FireworkRed,
                    item::Reagent::Yellow => comp::object::Body::FireworkYellow,
                },
                item::Throwable::TrainingDummy => comp::object::Body::TrainingDummy,
            })
            .with(comp::Pos(pos.0 + Vec3::unit_z() * 0.25))
            .with(comp::Vel(vel));

        match kind {
            item::Throwable::Bomb => {
                new_entity = new_entity.with(comp::Object::Bomb { owner: uid });
            },
            item::Throwable::Firework(reagent) => {
                new_entity = new_entity
                    .with(comp::Object::Firework {
                        owner: uid,
                        reagent,
                    })
                    .with(LightEmitter {
                        animated: true,
                        flicker: 2.0,
                        strength: 2.0,
                        col: Rgb::new(1.0, 1.0, 0.0),
                    });
            },
            item::Throwable::TrainingDummy => {
                new_entity = new_entity.with(comp::Stats::new(
                    "Training Dummy".to_string(),
                    comp::object::Body::TrainingDummy.into(),
                ));
            },
        };

        new_entity.build();
    }
}

fn within_pickup_range(player_position: Option<&Pos>, item_position: Option<&Pos>) -> bool {
    match (player_position, item_position) {
        (Some(ppos), Some(ipos)) => ppos.0.distance_squared(ipos.0) < MAX_PICKUP_RANGE.powi(2),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::comp::Pos;
    use vek::Vec3;

    #[test]
    fn pickup_distance_within_range() {
        let player_position = Pos(Vec3::zero());
        let item_position = Pos(Vec3::one());

        assert_eq!(
            within_pickup_range(Some(&player_position), Some(&item_position)),
            true
        );
    }

    #[test]
    fn pickup_distance_not_within_range() {
        let player_position = Pos(Vec3::zero());
        let item_position = Pos(Vec3::one() * 500.0);

        assert_eq!(
            within_pickup_range(Some(&player_position), Some(&item_position)),
            false
        );
    }
}
