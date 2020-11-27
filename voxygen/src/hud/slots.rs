use super::{
    hotbar::{self, Slot as HotbarSlot},
    img_ids,
    item_imgs::{ItemImgs, ItemKey},
};
use crate::ui::slot::{self, SlotKey, SumSlot};
use common::comp::{
    item::{
        tool::{AbilityMap, ToolKind},
        ItemKind,
    },
    Energy, Inventory, Loadout,
};
use conrod_core::{image, Color};

pub use common::comp::slot::{ArmorSlot, EquipSlot};

#[derive(Clone, Copy, PartialEq)]
pub enum SlotKind {
    Inventory(InventorySlot),
    Equip(EquipSlot),
    Hotbar(HotbarSlot),
    /* Spellbook(SpellbookSlot), TODO */
}

pub type SlotManager = slot::SlotManager<SlotKind>;

#[derive(Clone, Copy, PartialEq)]
pub struct InventorySlot(pub usize);

impl SlotKey<Inventory, ItemImgs> for InventorySlot {
    type ImageKey = ItemKey;

    fn image_key(&self, source: &Inventory) -> Option<(Self::ImageKey, Option<Color>)> {
        source.get(self.0).map(|i| (i.into(), None))
    }

    fn amount(&self, source: &Inventory) -> Option<u32> {
        source
            .get(self.0)
            .map(|item| item.amount())
            .filter(|amount| *amount > 1)
    }

    fn image_id(key: &Self::ImageKey, source: &ItemImgs) -> image::Id {
        source.img_id_or_not_found_img(key.clone())
    }
}

impl SlotKey<Loadout, ItemImgs> for EquipSlot {
    type ImageKey = ItemKey;

    fn image_key(&self, source: &Loadout) -> Option<(Self::ImageKey, Option<Color>)> {
        let item = match self {
            EquipSlot::Armor(ArmorSlot::Shoulders) => source.shoulder.as_ref(),
            EquipSlot::Armor(ArmorSlot::Chest) => source.chest.as_ref(),
            EquipSlot::Armor(ArmorSlot::Belt) => source.belt.as_ref(),
            EquipSlot::Armor(ArmorSlot::Hands) => source.hand.as_ref(),
            EquipSlot::Armor(ArmorSlot::Legs) => source.pants.as_ref(),
            EquipSlot::Armor(ArmorSlot::Feet) => source.foot.as_ref(),
            EquipSlot::Armor(ArmorSlot::Back) => source.back.as_ref(),
            EquipSlot::Armor(ArmorSlot::Ring) => source.ring.as_ref(),
            EquipSlot::Armor(ArmorSlot::Neck) => source.neck.as_ref(),
            EquipSlot::Armor(ArmorSlot::Head) => source.head.as_ref(),
            EquipSlot::Armor(ArmorSlot::Tabard) => source.tabard.as_ref(),
            EquipSlot::Mainhand => source.active_item.as_ref().map(|i| &i.item),
            EquipSlot::Offhand => source.second_item.as_ref().map(|i| &i.item),
            EquipSlot::Lantern => source.lantern.as_ref(),
            EquipSlot::Glider => source.glider.as_ref(),
        };

        item.map(|i| (i.into(), None))
    }

    fn amount(&self, _: &Loadout) -> Option<u32> { None }

    fn image_id(key: &Self::ImageKey, source: &ItemImgs) -> image::Id {
        source.img_id_or_not_found_img(key.clone())
    }
}

#[derive(Clone, PartialEq)]
pub enum HotbarImage {
    Item(ItemKey),
    FireAoe,
    SnakeArrow,
    SwordWhirlwind,
    HammerLeap,
    AxeLeapSlash,
    BowJumpBurst,
}

type HotbarSource<'a> = (
    &'a hotbar::State,
    &'a Inventory,
    &'a Loadout,
    &'a Energy,
    &'a AbilityMap,
);
type HotbarImageSource<'a> = (&'a ItemImgs, &'a img_ids::Imgs);

impl<'a> SlotKey<HotbarSource<'a>, HotbarImageSource<'a>> for HotbarSlot {
    type ImageKey = HotbarImage;

    fn image_key(
        &self,
        (hotbar, inventory, loadout, energy, ability_map): &HotbarSource<'a>,
    ) -> Option<(Self::ImageKey, Option<Color>)> {
        hotbar.get(*self).and_then(|contents| match contents {
            hotbar::SlotContents::Inventory(idx) => inventory
                .get(idx)
                .map(|item| HotbarImage::Item(item.into()))
                .map(|i| (i, None)),
            hotbar::SlotContents::Ability3 => {
                let tool = match loadout.active_item.as_ref().map(|i| i.item.kind()) {
                    Some(ItemKind::Tool(tool)) => Some(tool),
                    _ => None,
                };

                tool.and_then(|tool| {
                    match tool.kind {
                        ToolKind::Staff => Some(HotbarImage::FireAoe),
                        ToolKind::Hammer => Some(HotbarImage::HammerLeap),
                        ToolKind::Axe => Some(HotbarImage::AxeLeapSlash),
                        ToolKind::Bow => Some(HotbarImage::BowJumpBurst),
                        ToolKind::Debug => Some(HotbarImage::SnakeArrow),
                        ToolKind::Sword => Some(HotbarImage::SwordWhirlwind),
                        _ => None,
                    }
                    .map(|i| {
                        (
                            i,
                            if let Some(skill) = tool.get_abilities(ability_map).skills.get(0) {
                                if energy.current() >= skill.get_energy_cost() {
                                    Some(Color::Rgba(1.0, 1.0, 1.0, 1.0))
                                } else {
                                    Some(Color::Rgba(0.3, 0.3, 0.3, 0.8))
                                }
                            } else {
                                Some(Color::Rgba(1.0, 1.0, 1.0, 1.0))
                            },
                        )
                    })
                })
            },
        })
    }

    fn amount(&self, (hotbar, inventory, _, _, _): &HotbarSource<'a>) -> Option<u32> {
        hotbar
            .get(*self)
            .and_then(|content| match content {
                hotbar::SlotContents::Inventory(idx) => inventory.get(idx),
                hotbar::SlotContents::Ability3 => None,
            })
            .map(|item| item.amount())
            .filter(|amount| *amount > 1)
    }

    fn image_id(key: &Self::ImageKey, (item_imgs, imgs): &HotbarImageSource<'a>) -> image::Id {
        match key {
            HotbarImage::Item(key) => item_imgs.img_id_or_not_found_img(key.clone()),
            HotbarImage::SnakeArrow => imgs.snake_arrow_0,
            HotbarImage::FireAoe => imgs.fire_aoe,
            HotbarImage::SwordWhirlwind => imgs.sword_whirlwind,
            HotbarImage::HammerLeap => imgs.hammerleap,
            HotbarImage::AxeLeapSlash => imgs.skill_axe_leap_slash,
            HotbarImage::BowJumpBurst => imgs.skill_bow_jump_burst,
        }
    }
}

impl From<InventorySlot> for SlotKind {
    fn from(inventory: InventorySlot) -> Self { Self::Inventory(inventory) }
}

impl From<EquipSlot> for SlotKind {
    fn from(equip: EquipSlot) -> Self { Self::Equip(equip) }
}

impl From<HotbarSlot> for SlotKind {
    fn from(hotbar: HotbarSlot) -> Self { Self::Hotbar(hotbar) }
}

impl SumSlot for SlotKind {}
