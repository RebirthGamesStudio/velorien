use crate::{
    comp,
    comp::{
        item::{self, armor, tool::AbilityMap},
        ItemConfig,
    },
};
use comp::{Inventory, Loadout};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Slot {
    Inventory(usize),
    Equip(EquipSlot),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum EquipSlot {
    Armor(ArmorSlot),
    Mainhand,
    Offhand,
    Lantern,
    Glider,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Neck,
    Shoulders,
    Chest,
    Hands,
    Ring,
    Back,
    Belt,
    Legs,
    Feet,
    Tabard,
}

//const ALL_ARMOR_SLOTS: [ArmorSlot; 11] = [
//    Head, Neck, Shoulders, Chest, Hands, Ring, Back, Belt, Legs, Feet, Tabard,
//];

impl Slot {
    pub fn can_hold(self, item_kind: &item::ItemKind) -> bool {
        match (self, item_kind) {
            (Self::Inventory(_), _) => true,
            (Self::Equip(slot), item_kind) => slot.can_hold(item_kind),
        }
    }
}

impl EquipSlot {
    fn can_hold(self, item_kind: &item::ItemKind) -> bool {
        use armor::Armor;
        use item::ItemKind;
        match (self, item_kind) {
            (Self::Armor(slot), ItemKind::Armor(Armor { kind, .. })) => slot.can_hold(kind),
            (Self::Mainhand, ItemKind::Tool(_)) => true,
            (Self::Offhand, ItemKind::Tool(_)) => true,
            (Self::Lantern, ItemKind::Lantern(_)) => true,
            (Self::Glider, ItemKind::Glider(_)) => true,
            _ => false,
        }
    }
}

impl ArmorSlot {
    fn can_hold(self, armor: &item::armor::ArmorKind) -> bool {
        use item::armor::ArmorKind;
        matches!(
            (self, armor),
            (Self::Head, ArmorKind::Head(_))
                | (Self::Neck, ArmorKind::Neck(_))
                | (Self::Shoulders, ArmorKind::Shoulder(_))
                | (Self::Chest, ArmorKind::Chest(_))
                | (Self::Hands, ArmorKind::Hand(_))
                | (Self::Ring, ArmorKind::Ring(_))
                | (Self::Back, ArmorKind::Back(_))
                | (Self::Belt, ArmorKind::Belt(_))
                | (Self::Legs, ArmorKind::Pants(_))
                | (Self::Feet, ArmorKind::Foot(_))
                | (Self::Tabard, ArmorKind::Tabard(_))
        )
    }
}

/// Replace an equipment slot with an item. Return the item that was in the
/// slot, if any. Doesn't update the inventory.
fn loadout_replace(
    equip_slot: EquipSlot,
    item: Option<item::Item>,
    loadout: &mut Loadout,
    map: &AbilityMap,
) -> Option<item::Item> {
    use std::mem::replace;
    match equip_slot {
        EquipSlot::Armor(ArmorSlot::Head) => replace(&mut loadout.head, item),
        EquipSlot::Armor(ArmorSlot::Neck) => replace(&mut loadout.neck, item),
        EquipSlot::Armor(ArmorSlot::Shoulders) => replace(&mut loadout.shoulder, item),
        EquipSlot::Armor(ArmorSlot::Chest) => replace(&mut loadout.chest, item),
        EquipSlot::Armor(ArmorSlot::Hands) => replace(&mut loadout.hand, item),
        EquipSlot::Armor(ArmorSlot::Ring) => replace(&mut loadout.ring, item),
        EquipSlot::Armor(ArmorSlot::Back) => replace(&mut loadout.back, item),
        EquipSlot::Armor(ArmorSlot::Belt) => replace(&mut loadout.belt, item),
        EquipSlot::Armor(ArmorSlot::Legs) => replace(&mut loadout.pants, item),
        EquipSlot::Armor(ArmorSlot::Feet) => replace(&mut loadout.foot, item),
        EquipSlot::Armor(ArmorSlot::Tabard) => replace(&mut loadout.tabard, item),
        EquipSlot::Lantern => replace(&mut loadout.lantern, item),
        EquipSlot::Glider => replace(&mut loadout.glider, item),
        EquipSlot::Mainhand => replace(
            &mut loadout.active_item,
            item.map(|item| ItemConfig::from((item, map))),
        )
        .map(|i| i.item),
        EquipSlot::Offhand => replace(
            &mut loadout.second_item,
            item.map(|item| ItemConfig::from((item, map))),
        )
        .map(|i| i.item),
    }
}

/// Insert an item into a loadout. If the specified slot is already occupied
/// the old item is returned.
#[must_use]
fn loadout_insert(
    equip_slot: EquipSlot,
    item: item::Item,
    loadout: &mut Loadout,
    map: &AbilityMap,
) -> Option<item::Item> {
    loadout_replace(equip_slot, Some(item), loadout, map)
}

/// Remove an item from a loadout.
///
/// ```
/// use veloren_common::{
///     assets::AssetExt,
///     comp::{
///         item::tool::AbilityMap,
///         slot::{loadout_remove, EquipSlot},
///         Inventory,
///     },
///     LoadoutBuilder,
/// };
///
/// let mut inv = Inventory::new_empty();
///
/// let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");
///
/// let mut loadout = LoadoutBuilder::new()
///     .defaults()
///     .active_item(Some(LoadoutBuilder::default_item_config_from_str(
///         "common.items.weapons.sword.zweihander_sword_0",
///         &map,
///     )))
///     .build();
///
/// let slot = EquipSlot::Mainhand;
///
/// loadout_remove(slot, &mut loadout, &map);
/// assert_eq!(None, loadout.active_item);
/// ```
pub fn loadout_remove(
    equip_slot: EquipSlot,
    loadout: &mut Loadout,
    map: &AbilityMap,
) -> Option<item::Item> {
    loadout_replace(equip_slot, None, loadout, map)
}

/// Swap item in an inventory slot with one in a loadout slot.
fn swap_inventory_loadout(
    inventory_slot: usize,
    equip_slot: EquipSlot,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    map: &AbilityMap,
) {
    // Check if loadout slot can hold item
    if inventory
        .get(inventory_slot)
        .map_or(true, |item| equip_slot.can_hold(&item.kind()))
    {
        // Take item from loadout
        let from_equip = loadout_remove(equip_slot, loadout, map);
        // Swap with item in the inventory
        let from_inv = if let Some(item) = from_equip {
            // If this fails and we get item back as an err it will just be put back in the
            // loadout
            inventory.insert(inventory_slot, item).unwrap_or_else(Some)
        } else {
            inventory.remove(inventory_slot)
        };
        // Put item from the inventory in loadout
        if let Some(item) = from_inv {
            loadout_insert(equip_slot, item, loadout, map).unwrap_none(); // Can never fail
        }
    }
}

/// Swap items in loadout. Does nothing if items are not compatible with their
/// new slots.
fn swap_loadout(slot_a: EquipSlot, slot_b: EquipSlot, loadout: &mut Loadout, map: &AbilityMap) {
    // Ensure that the slots are not the same
    if slot_a == slot_b {
        warn!("Tried to swap equip slot with itself");
        return;
    }

    // Get items from the slots
    let item_a = loadout_remove(slot_a, loadout, map);
    let item_b = loadout_remove(slot_b, loadout, map);
    // Check if items can go in the other slots
    if item_a.as_ref().map_or(true, |i| slot_b.can_hold(&i.kind()))
        && item_b.as_ref().map_or(true, |i| slot_a.can_hold(&i.kind()))
    {
        // Swap
        loadout_replace(slot_b, item_a, loadout, map).unwrap_none();
        loadout_replace(slot_a, item_b, loadout, map).unwrap_none();
    } else {
        // Otherwise put the items back
        loadout_replace(slot_a, item_a, loadout, map).unwrap_none();
        loadout_replace(slot_b, item_b, loadout, map).unwrap_none();
    }
}

// TODO: Should this report if a change actually occurred? (might be useful when
// minimizing network use)

/// Swap items from two slots, regardless of if either is inventory or loadout.
pub fn swap(
    slot_a: Slot,
    slot_b: Slot,
    inventory: Option<&mut Inventory>,
    loadout: Option<&mut Loadout>,
    map: &AbilityMap,
) {
    match (slot_a, slot_b) {
        (Slot::Inventory(slot_a), Slot::Inventory(slot_b)) => {
            inventory.map(|i| i.swap_slots(slot_a, slot_b));
        },
        (Slot::Inventory(inv_slot), Slot::Equip(equip_slot))
        | (Slot::Equip(equip_slot), Slot::Inventory(inv_slot)) => {
            if let Some((inventory, loadout)) = loadout.and_then(|l| inventory.map(|i| (i, l))) {
                swap_inventory_loadout(inv_slot, equip_slot, inventory, loadout, map);
            }
        },

        (Slot::Equip(slot_a), Slot::Equip(slot_b)) => {
            loadout.map(|l| swap_loadout(slot_a, slot_b, l, map));
        },
    }
}

/// Equip an item from a slot in inventory. The currently equipped item will go
/// into inventory. If the item is going to mainhand, put mainhand in
/// offhand and place offhand into inventory.
///
/// ```
/// use veloren_common::{
///     assets::AssetExt,
///     comp::{
///         item::tool::AbilityMap,
///         slot::{equip, EquipSlot},
///         Inventory, Item,
///     },
///     LoadoutBuilder,
/// };
///
/// let boots = Item::new_from_asset_expect("common.items.testing.test_boots");
///
/// let mut inv = Inventory::new_empty();
/// inv.push(boots.duplicate());
///
/// let mut loadout = LoadoutBuilder::new().defaults().build();
///
/// let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");
///
/// equip(0, &mut inv, &mut loadout, &map);
/// assert_eq!(Some(boots), loadout.foot);
/// ```
pub fn equip(slot: usize, inventory: &mut Inventory, loadout: &mut Loadout, map: &AbilityMap) {
    use armor::Armor;
    use item::{armor::ArmorKind, ItemKind};

    let equip_slot = inventory.get(slot).and_then(|i| match &i.kind() {
        ItemKind::Tool(_) => Some(EquipSlot::Mainhand),
        ItemKind::Armor(Armor { kind, .. }) => Some(EquipSlot::Armor(match kind {
            ArmorKind::Head(_) => ArmorSlot::Head,
            ArmorKind::Neck(_) => ArmorSlot::Neck,
            ArmorKind::Shoulder(_) => ArmorSlot::Shoulders,
            ArmorKind::Chest(_) => ArmorSlot::Chest,
            ArmorKind::Hand(_) => ArmorSlot::Hands,
            ArmorKind::Ring(_) => ArmorSlot::Ring,
            ArmorKind::Back(_) => ArmorSlot::Back,
            ArmorKind::Belt(_) => ArmorSlot::Belt,
            ArmorKind::Pants(_) => ArmorSlot::Legs,
            ArmorKind::Foot(_) => ArmorSlot::Feet,
            ArmorKind::Tabard(_) => ArmorSlot::Tabard,
        })),
        ItemKind::Lantern(_) => Some(EquipSlot::Lantern),
        ItemKind::Glider(_) => Some(EquipSlot::Glider),
        _ => None,
    });

    if let Some(equip_slot) = equip_slot {
        // If item is going to mainhand, put mainhand in offhand and place offhand in
        // inventory
        if let EquipSlot::Mainhand = equip_slot {
            swap_loadout(EquipSlot::Mainhand, EquipSlot::Offhand, loadout, map);
        }

        swap_inventory_loadout(slot, equip_slot, inventory, loadout, map);
    }
}

/// Unequip an item from slot and place into inventory. Will leave the item
/// equipped if inventory has no slots available.
///
/// ```
/// use veloren_common::{
///     assets::AssetExt,
///     comp::{
///         item::tool::AbilityMap,
///         slot::{unequip, EquipSlot},
///         Inventory,
///     },
///     LoadoutBuilder,
/// };
///
/// let mut inv = Inventory::new_empty();
///
/// let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");
///
/// let mut loadout = LoadoutBuilder::new()
///     .defaults()
///     .active_item(Some(LoadoutBuilder::default_item_config_from_str(
///         "common.items.weapons.sword.zweihander_sword_0",
///         &map,
///     )))
///     .build();
///
/// let slot = EquipSlot::Mainhand;
///
/// unequip(slot, &mut inv, &mut loadout, &map);
/// assert_eq!(None, loadout.active_item);
/// ```
pub fn unequip(
    slot: EquipSlot,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    map: &AbilityMap,
) {
    loadout_remove(slot, loadout, map) // Remove item from loadout
        .and_then(|i| inventory.push(i)) // Insert into inventory
        .and_then(|i| loadout_insert(slot, i, loadout, map)) // If that fails put back in loadout
        .unwrap_none(); // Never fails
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::AssetExt, comp::Item, LoadoutBuilder};

    #[test]
    fn test_unequip_items_both_hands() {
        let mut inv = Inventory {
            slots: vec![None],
            amount: 0,
        };

        let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");

        let sword = LoadoutBuilder::default_item_config_from_str(
            "common.items.weapons.sword.zweihander_sword_0",
            &map,
        );

        let mut loadout = LoadoutBuilder::new()
            .defaults()
            .active_item(Some(sword.clone()))
            .second_item(Some(sword.clone()))
            .build();

        assert_eq!(Some(sword.clone()), loadout.active_item);
        unequip(EquipSlot::Mainhand, &mut inv, &mut loadout, &map);
        // We have space in the inventory, so this should have unequipped
        assert_eq!(None, loadout.active_item);

        unequip(EquipSlot::Offhand, &mut inv, &mut loadout, &map);
        // There is no more space in the inventory, so this should still be equipped
        assert_eq!(Some(sword.clone()), loadout.second_item);

        // Verify inventory
        assert_eq!(inv.slots[0], Some(sword.item));
        assert_eq!(inv.slots.len(), 1);
    }

    #[test]
    fn test_equip_item() {
        let boots: Option<comp::Item> = Some(Item::new_from_asset_expect(
            "common.items.testing.test_boots",
        ));

        let starting_sandles: Option<comp::Item> = Some(Item::new_from_asset_expect(
            "common.items.armor.starter.sandals_0",
        ));

        let mut inv = Inventory {
            slots: vec![boots.clone()],
            amount: 1,
        };

        let mut loadout = LoadoutBuilder::new().defaults().build();

        let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");

        // We should start with the starting sandles
        assert_eq!(starting_sandles, loadout.foot);
        equip(0, &mut inv, &mut loadout, &map);

        // We should now have the testing boots equiped
        assert_eq!(boots, loadout.foot);

        // Verify inventory
        assert_eq!(inv.slots[0], starting_sandles);
        assert_eq!(inv.slots.len(), 1);
    }

    #[test]
    fn test_loadout_replace() {
        let boots: Option<comp::Item> = Some(Item::new_from_asset_expect(
            "common.items.testing.test_boots",
        ));

        let starting_sandles: Option<comp::Item> = Some(Item::new_from_asset_expect(
            "common.items.armor.starter.sandals_0",
        ));

        let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");

        let mut loadout = LoadoutBuilder::new().defaults().build();

        // We should start with the starting sandles
        assert_eq!(starting_sandles, loadout.foot);

        // The swap should return the sandles
        assert_eq!(
            starting_sandles,
            loadout_replace(
                EquipSlot::Armor(ArmorSlot::Feet),
                boots.clone(),
                &mut loadout,
                &map,
            )
        );

        // We should now have the testing boots equiped
        assert_eq!(boots, loadout.foot);
    }

    #[test]
    fn test_loadout_remove() {
        let map = AbilityMap::load_expect_cloned("common.abilities.weapon_ability_manifest");

        let sword = LoadoutBuilder::default_item_config_from_str(
            "common.items.weapons.sword.zweihander_sword_0",
            &map,
        );

        let mut loadout = LoadoutBuilder::new()
            .defaults()
            .active_item(Some(sword.clone()))
            .build();

        // The swap should return the sword
        assert_eq!(
            Some(sword.item),
            loadout_remove(EquipSlot::Mainhand, &mut loadout, &map)
        );

        // We should now have nothing equiped
        assert_eq!(None, loadout.active_item);
    }
}
