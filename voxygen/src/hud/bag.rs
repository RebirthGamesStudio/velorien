use super::{
    img_ids::{Imgs, ImgsRot},
    item_imgs::ItemImgs,
    slots::{ArmorSlot, EquipSlot, InventorySlot, SlotManager},
    util::loadout_slot_text,
    Show, CRITICAL_HP_COLOR, LOW_HP_COLOR, QUALITY_COMMON, TEXT_COLOR, UI_HIGHLIGHT_0, UI_MAIN,
    XP_COLOR,
};
use crate::{
    hud::get_quality_col,
    i18n::Localization,
    ui::{
        fonts::Fonts,
        slot::{ContentSize, SlotMaker},
        ImageFrame, Tooltip, TooltipManager, Tooltipable,
    },
};
use client::Client;
use common::comp::{item::Quality, Stats};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Color, Colorable, Labelable, Positionable, Sizeable, Widget, WidgetCommon,
};

use vek::Vec2;

widget_ids! {
    pub struct Ids {
        test,
        bag_close,
        inv_alignment,
        inv_grid_1,
        inv_grid_2,
        inv_scrollbar,
        inv_slots_0,
        inv_slots[],
        //tooltip[],
        bg,
        bg_frame,
        char_ico,
        coin_ico,
        space_txt,
        currency_txt,
        inventory_title,
        inventory_title_bg,
        scrollbar_bg,
        stats_button,
        tab_1,
        tab_2,
        tab_3,
        tab_4,
        // Stats
        stats_alignment,
        level,
        exp_rectangle,
        exp_progress_rectangle,
        expbar,
        exp,
        divider,
        statnames,
        stats,
        // Armor Slots
        slots_bg,
        head_slot,
        neck_slot,
        chest_slot,
        shoulders_slot,
        hands_slot,
        legs_slot,
        belt_slot,
        lantern_slot,
        ring_slot,
        feet_slot,
        back_slot,
        tabard_slot,
        glider_slot,
        mainhand_slot,
        offhand_slot,
        // ???
        end_ico,
        fit_ico,
        wp_ico,
        prot_ico,
    }
}

#[derive(WidgetCommon)]
pub struct Bag<'a> {
    client: &'a Client,
    imgs: &'a Imgs,
    item_imgs: &'a ItemImgs,
    fonts: &'a Fonts,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    rot_imgs: &'a ImgsRot,
    tooltip_manager: &'a mut TooltipManager,
    slot_manager: &'a mut SlotManager,
    _pulse: f32,
    localized_strings: &'a Localization,

    stats: &'a Stats,
    show: &'a Show,
}

impl<'a> Bag<'a> {
    #[allow(clippy::too_many_arguments)] // TODO: Pending review in #587
    pub fn new(
        client: &'a Client,
        imgs: &'a Imgs,
        item_imgs: &'a ItemImgs,
        fonts: &'a Fonts,
        rot_imgs: &'a ImgsRot,
        tooltip_manager: &'a mut TooltipManager,
        slot_manager: &'a mut SlotManager,
        pulse: f32,
        localized_strings: &'a Localization,
        stats: &'a Stats,
        show: &'a Show,
    ) -> Self {
        Self {
            client,
            imgs,
            item_imgs,
            fonts,
            common: widget::CommonBuilder::default(),
            rot_imgs,
            tooltip_manager,
            slot_manager,
            _pulse: pulse,
            localized_strings,
            stats,
            show,
        }
    }
}

pub struct State {
    ids: Ids,
}

pub enum Event {
    Stats,
    Close,
}

impl<'a> Widget for Bag<'a> {
    type Event = Option<Event>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    #[allow(clippy::unused_unit)] // TODO: Pending review in #587
    fn style(&self) -> Self::Style { () }

    #[allow(clippy::useless_format)] // TODO: Pending review in #587
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, .. } = args;

        let mut event = None;

        let invs = self.client.inventories();
        let inventory = match invs.get(self.client.entity()) {
            Some(i) => i,
            None => return None,
        };
        let loadouts = self.client.loadouts();
        let loadout = match loadouts.get(self.client.entity()) {
            Some(l) => l,
            None => return None,
        };
        let exp_percentage = (self.stats.exp.current() as f64) / (self.stats.exp.maximum() as f64);
        let exp_threshold = format!(
            "{}/{} {}",
            self.stats.exp.current(),
            self.stats.exp.maximum(),
            &self.localized_strings.get("hud.bag.exp")
        );
        let space_used = inventory.amount();
        let space_max = inventory.slots().len();
        let bag_space = format!("{}/{}", space_used, space_max);
        let bag_space_percentage = space_used as f32 / space_max as f32;
        let level = (self.stats.level.level()).to_string();
        let currency = 0; // TODO: Add as a Stat          

        // Tooltips
        let item_tooltip = Tooltip::new({
            // Edge images [t, b, r, l]
            // Corner images [tr, tl, br, bl]
            let edge = &self.rot_imgs.tt_side;
            let corner = &self.rot_imgs.tt_corner;
            ImageFrame::new(
                [edge.cw180, edge.none, edge.cw270, edge.cw90],
                [corner.none, corner.cw270, corner.cw90, corner.cw180],
                Color::Rgba(0.08, 0.07, 0.04, 1.0),
                5.0,
            )
        })
        .title_font_size(self.fonts.cyri.scale(15))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(12))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);
        // BG
        Image::new(if self.show.stats {
            self.imgs.inv_bg_stats
        } else {
            self.imgs.inv_bg_armor
        })
        .w_h(424.0, 708.0)
        .bottom_right_with_margins_on(ui.window, 60.0, 5.0)
        .color(Some(UI_MAIN))
        .set(state.ids.bg, ui);
        Image::new(self.imgs.inv_frame)
            .w_h(424.0, 708.0)
            .middle_of(state.ids.bg)
            .color(Some(UI_HIGHLIGHT_0))
            .set(state.ids.bg_frame, ui);
        // Title
        Text::new(
            &self
                .localized_strings
                .get("hud.bag.inventory")
                .replace("{playername}", &self.stats.name.to_string().as_str()),
        )
        .mid_top_with_margin_on(state.ids.bg_frame, 9.0)
        .font_id(self.fonts.cyri.conrod_id)
        .font_size(self.fonts.cyri.scale(20))
        .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
        .set(state.ids.inventory_title_bg, ui);
        Text::new(
            &self
                .localized_strings
                .get("hud.bag.inventory")
                .replace("{playername}", &self.stats.name.to_string().as_str()),
        )
        .top_left_with_margins_on(state.ids.inventory_title_bg, 2.0, 2.0)
        .font_id(self.fonts.cyri.conrod_id)
        .font_size(self.fonts.cyri.scale(20))
        .color(TEXT_COLOR)
        .set(state.ids.inventory_title, ui);
        // Scrollbar-BG
        Image::new(self.imgs.scrollbar_bg)
            .w_h(9.0, 173.0)
            .bottom_right_with_margins_on(state.ids.bg_frame, 42.0, 3.0)
            .color(Some(UI_HIGHLIGHT_0))
            .set(state.ids.scrollbar_bg, ui);
        // Char Pixel-Art
        Image::new(self.imgs.char_art)
            .w_h(40.0, 37.0)
            .top_left_with_margins_on(state.ids.bg, 4.0, 2.0)
            .set(state.ids.char_ico, ui);
        // Coin Icon and Currency Text
        Image::new(self.imgs.coin_ico)
            .w_h(16.0, 17.0)
            .bottom_left_with_margins_on(state.ids.bg_frame, 2.0, 43.0)
            .set(state.ids.coin_ico, ui);
        Text::new(&format!("{}", currency))
            .bottom_left_with_margins_on(state.ids.bg_frame, 6.0, 64.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(14))
            .color(Color::Rgba(0.871, 0.863, 0.05, 1.0))
            .set(state.ids.currency_txt, ui);
        //Free Bag-Space
        Text::new(&bag_space)
            .bottom_right_with_margins_on(state.ids.bg_frame, 6.0, 43.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(14))
            .color(if bag_space_percentage < 0.8 {
                TEXT_COLOR
            } else if bag_space_percentage < 1.0 {
                LOW_HP_COLOR
            } else {
                CRITICAL_HP_COLOR
            })
            .set(state.ids.space_txt, ui);
        // Alignment for Grid
        Rectangle::fill_with([362.0, 200.0], color::TRANSPARENT)
            .bottom_left_with_margins_on(state.ids.bg_frame, 29.0, 44.0)
            .scroll_kids_vertically()
            .set(state.ids.inv_alignment, ui);

        if !self.show.stats {
            // Title
            Text::new(
                &self
                    .localized_strings
                    .get("hud.bag.inventory")
                    .replace("{playername}", &self.stats.name.to_string().as_str()),
            )
            .mid_top_with_margin_on(state.ids.bg_frame, 9.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(22))
            .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
            .set(state.ids.inventory_title_bg, ui);
            Text::new(
                &self
                    .localized_strings
                    .get("hud.bag.inventory")
                    .replace("{playername}", &self.stats.name.to_string().as_str()),
            )
            .top_left_with_margins_on(state.ids.inventory_title_bg, 2.0, 2.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(22))
            .color(TEXT_COLOR)
            .set(state.ids.inventory_title, ui);
            // Armor Slots
            let mut slot_maker = SlotMaker {
                empty_slot: self.imgs.armor_slot_empty,
                filled_slot: self.imgs.armor_slot,
                selected_slot: self.imgs.armor_slot_sel,
                background_color: Some(UI_HIGHLIGHT_0),
                content_size: ContentSize {
                    width_height_ratio: 1.0,
                    max_fraction: 0.75, /* Changes the item image size by setting a maximum
                                         * fraction
                                         * of either the width or height */
                },
                selected_content_scale: 1.067,
                amount_font: self.fonts.cyri.conrod_id,
                amount_margins: Vec2::new(-4.0, 0.0),
                amount_font_size: self.fonts.cyri.scale(12),
                amount_text_color: TEXT_COLOR,
                content_source: loadout,
                image_source: self.item_imgs,
                slot_manager: Some(self.slot_manager),
            };
            let i18n = &self.localized_strings;
            let filled_slot = self.imgs.armor_slot;
            //  Head
            let (title, desc) =
                loadout_slot_text(loadout.head.as_ref(), || (i18n.get("hud.bag.head"), ""));
            let head_q_col = loadout
                .head
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Head), [45.0; 2])
                .mid_top_with_margin_on(state.ids.bg_frame, 60.0)
                .with_icon(self.imgs.head_bg, Vec2::new(32.0, 40.0), Some(UI_MAIN))
                .with_background_color(TEXT_COLOR)
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    head_q_col,
                )
                .set(state.ids.head_slot, ui);
            //  Necklace
            let (title, desc) =
                loadout_slot_text(loadout.neck.as_ref(), || (i18n.get("hud.bag.neck"), ""));
            let neck_q_col = loadout
                .neck
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Neck), [45.0; 2])
                .mid_bottom_with_margin_on(state.ids.head_slot, -55.0)
                .with_icon(self.imgs.necklace_bg, Vec2::new(40.0, 31.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    neck_q_col,
                )
                .set(state.ids.neck_slot, ui);
            // Chest
            //Image::new(self.imgs.armor_slot) // different graphics for empty/non empty
            let (title, desc) =
                loadout_slot_text(loadout.chest.as_ref(), || (i18n.get("hud.bag.chest"), ""));
            let chest_q_col = loadout
                .chest
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Chest), [85.0; 2])
                .mid_bottom_with_margin_on(state.ids.neck_slot, -95.0)
                .with_icon(self.imgs.chest_bg, Vec2::new(64.0, 42.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    chest_q_col,
                )
                .set(state.ids.chest_slot, ui);
            //  Shoulders
            let (title, desc) = loadout_slot_text(loadout.shoulder.as_ref(), || {
                (i18n.get("hud.bag.shoulders"), "")
            });
            let shoulder_q_col = loadout
                .shoulder
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Shoulders), [70.0; 2])
                .bottom_left_with_margins_on(state.ids.chest_slot, 0.0, -80.0)
                .with_icon(self.imgs.shoulders_bg, Vec2::new(60.0, 36.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    shoulder_q_col,
                )
                .set(state.ids.shoulders_slot, ui);
            // Hands
            let (title, desc) =
                loadout_slot_text(loadout.hand.as_ref(), || (i18n.get("hud.bag.hands"), ""));
            let chest_q_col = loadout
                .hand
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Hands), [70.0; 2])
                .bottom_right_with_margins_on(state.ids.chest_slot, 0.0, -80.0)
                .with_icon(self.imgs.hands_bg, Vec2::new(55.0, 60.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    chest_q_col,
                )
                .set(state.ids.hands_slot, ui);
            // Belt
            let (title, desc) =
                loadout_slot_text(loadout.belt.as_ref(), || (i18n.get("hud.bag.belt"), ""));
            let belt_q_col = loadout
                .belt
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Belt), [45.0; 2])
                .mid_bottom_with_margin_on(state.ids.chest_slot, -55.0)
                .with_icon(self.imgs.belt_bg, Vec2::new(40.0, 23.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    belt_q_col,
                )
                .set(state.ids.belt_slot, ui);
            // Legs
            let (title, desc) =
                loadout_slot_text(loadout.pants.as_ref(), || (i18n.get("hud.bag.legs"), ""));
            let legs_q_col = loadout
                .pants
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Legs), [85.0; 2])
                .mid_bottom_with_margin_on(state.ids.belt_slot, -95.0)
                .with_icon(self.imgs.legs_bg, Vec2::new(48.0, 70.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    legs_q_col,
                )
                .set(state.ids.legs_slot, ui);
            // Lantern
            let (title, desc) = loadout_slot_text(loadout.lantern.as_ref(), || {
                (i18n.get("hud.bag.lantern"), "")
            });
            let lantern_q_col = loadout
                .lantern
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Lantern, [45.0; 2])
                .bottom_right_with_margins_on(state.ids.shoulders_slot, -55.0, 0.0)
                .with_icon(self.imgs.lantern_bg, Vec2::new(24.0, 38.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    lantern_q_col,
                )
                .set(state.ids.lantern_slot, ui);
            // Ring
            let (title, desc) =
                loadout_slot_text(loadout.ring.as_ref(), || (i18n.get("hud.bag.ring"), ""));
            let ring_q_col = loadout
                .ring
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Ring), [45.0; 2])
                .bottom_left_with_margins_on(state.ids.hands_slot, -55.0, 0.0)
                .with_icon(self.imgs.ring_bg, Vec2::new(36.0, 40.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    ring_q_col,
                )
                .set(state.ids.ring_slot, ui);
            // Back
            let (title, desc) =
                loadout_slot_text(loadout.back.as_ref(), || (i18n.get("hud.bag.back"), ""));
            let back_q_col = loadout
                .back
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Back), [45.0; 2])
                .down_from(state.ids.lantern_slot, 10.0)
                .with_icon(self.imgs.back_bg, Vec2::new(33.0, 40.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    back_q_col,
                )
                .set(state.ids.back_slot, ui);
            // Foot
            let (title, desc) =
                loadout_slot_text(loadout.foot.as_ref(), || (i18n.get("hud.bag.feet"), ""));
            let foot_q_col = loadout
                .foot
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Feet), [45.0; 2])
                .down_from(state.ids.ring_slot, 10.0)
                .with_icon(self.imgs.feet_bg, Vec2::new(32.0, 40.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    foot_q_col,
                )
                .set(state.ids.feet_slot, ui);
            // Tabard
            let (title, desc) =
                loadout_slot_text(loadout.tabard.as_ref(), || (i18n.get("hud.bag.tabard"), ""));
            let tabard_q_col = loadout
                .tabard
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Armor(ArmorSlot::Tabard), [70.0; 2])
                .top_right_with_margins_on(state.ids.bg_frame, 80.5, 53.0)
                .with_icon(self.imgs.tabard_bg, Vec2::new(60.0, 60.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    tabard_q_col,
                )
                .set(state.ids.tabard_slot, ui);
            // Glider
            let (title, desc) =
                loadout_slot_text(loadout.glider.as_ref(), || (i18n.get("hud.bag.glider"), ""));
            let glider_q_col = loadout
                .glider
                .as_ref()
                .map(|item| get_quality_col(item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Glider, [70.0; 2])
                .top_left_with_margins_on(state.ids.bg_frame, 80.5, 53.0)
                .with_icon(self.imgs.glider_bg, Vec2::new(60.0, 60.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    glider_q_col,
                )
                .set(state.ids.glider_slot, ui);
            // Mainhand/Left-Slot
            let (title, desc) =
                loadout_slot_text(loadout.active_item.as_ref().map(|i| &i.item), || {
                    (i18n.get("hud.bag.mainhand"), "")
                });
            let mainhand_q_col = loadout
                .active_item
                .as_ref()
                .map(|item| get_quality_col(&item.item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Mainhand, [85.0; 2])
                .bottom_right_with_margins_on(state.ids.back_slot, -95.0, 0.0)
                .with_icon(self.imgs.mainhand_bg, Vec2::new(75.0, 75.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    mainhand_q_col,
                )
                .set(state.ids.mainhand_slot, ui);
            // Offhand/Right-Slot
            let (title, desc) =
                loadout_slot_text(loadout.second_item.as_ref().map(|i| &i.item), || {
                    (i18n.get("hud.bag.offhand"), "")
                });
            let offhand_q_col = loadout
                .second_item
                .as_ref()
                .map(|item| get_quality_col(&item.item))
                .unwrap_or(QUALITY_COMMON);
            slot_maker
                .fabricate(EquipSlot::Offhand, [85.0; 2])
                .bottom_left_with_margins_on(state.ids.feet_slot, -95.0, 0.0)
                .with_icon(self.imgs.offhand_bg, Vec2::new(75.0, 75.0), Some(UI_MAIN))
                .filled_slot(filled_slot)
                .with_tooltip(
                    self.tooltip_manager,
                    title,
                    &*desc,
                    &item_tooltip,
                    offhand_q_col,
                )
                .set(state.ids.offhand_slot, ui);
        } else {
            // Stats
            // Title
            Text::new(
                &self
                    .localized_strings
                    .get("hud.bag.stats_title")
                    .replace("{playername}", &self.stats.name.to_string().as_str()),
            )
            .mid_top_with_margin_on(state.ids.bg_frame, 9.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(22))
            .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
            .set(state.ids.inventory_title_bg, ui);
            Text::new(
                &self
                    .localized_strings
                    .get("hud.bag.stats_title")
                    .replace("{playername}", &self.stats.name.to_string().as_str()),
            )
            .top_left_with_margins_on(state.ids.inventory_title_bg, 2.0, 2.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(22))
            .color(TEXT_COLOR)
            .set(state.ids.inventory_title, ui);
            // Alignment for Stats
            Rectangle::fill_with([418.0, 384.0], color::TRANSPARENT)
                .mid_top_with_margin_on(state.ids.bg_frame, 48.0)
                .scroll_kids_vertically()
                .set(state.ids.stats_alignment, ui);
            // Level
            Text::new(&level)
                .mid_top_with_margin_on(state.ids.stats_alignment, 10.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(30))
                .color(TEXT_COLOR)
                .set(state.ids.level, ui);

            // Exp-Bar Background
            Rectangle::fill_with([170.0, 10.0], color::BLACK)
                .mid_top_with_margin_on(state.ids.stats_alignment, 50.0)
                .set(state.ids.exp_rectangle, ui);

            // Exp-Bar Progress
            Rectangle::fill_with([170.0 * (exp_percentage), 6.0], XP_COLOR) // 0.8 = Experience percentage
                .mid_left_with_margin_on(state.ids.expbar, 1.0)
                .set(state.ids.exp_progress_rectangle, ui);

            // Exp-Bar Foreground Frame
            Image::new(self.imgs.progress_frame)
                .w_h(170.0, 10.0)
                .middle_of(state.ids.exp_rectangle)
                .color(Some(UI_HIGHLIGHT_0))
                .set(state.ids.expbar, ui);

            // Exp-Text
            Text::new(&exp_threshold)
                .mid_top_with_margin_on(state.ids.expbar, 10.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(15))
                .color(TEXT_COLOR)
                .set(state.ids.exp, ui);

            // Divider
            /*Image::new(self.imgs.divider)
            .w_h(50.0, 5.0)
            .mid_top_with_margin_on(state.ids.exp, 20.0)
            .color(Some(UI_HIGHLIGHT_0))
            .set(state.ids.divider, ui);*/

            // Stats
            // Defense
            let damage_reduction = (100.0 * loadout.get_damage_reduction()) as i32;

            Text::new(
                &self
                    .localized_strings
                    .get("character_window.character_stats"),
            )
            .top_left_with_margins_on(state.ids.stats_alignment, 120.0, 150.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(16))
            .color(TEXT_COLOR)
            .set(state.ids.statnames, ui);
            Image::new(self.imgs.endurance_ico)
                .w_h(20.0, 20.0)
                .top_left_with_margins_on(state.ids.statnames, 0.0, -40.0)
                .color(Some(UI_HIGHLIGHT_0))
                .set(state.ids.end_ico, ui);
            Image::new(self.imgs.fitness_ico)
                .w_h(20.0, 20.0)
                .down_from(state.ids.end_ico, 15.0)
                .color(Some(UI_HIGHLIGHT_0))
                .set(state.ids.fit_ico, ui);
            Image::new(self.imgs.willpower_ico)
                .w_h(20.0, 20.0)
                .down_from(state.ids.fit_ico, 15.0)
                .color(Some(UI_HIGHLIGHT_0))
                .set(state.ids.wp_ico, ui);
            Image::new(self.imgs.protection_ico)
                .w_h(20.0, 20.0)
                .down_from(state.ids.wp_ico, 15.0)
                .color(Some(UI_HIGHLIGHT_0))
                .set(state.ids.prot_ico, ui);

            Text::new(&format!(
                "{}\n\n{}\n\n{}\n\n{}%",
                self.stats.endurance, self.stats.fitness, self.stats.willpower, damage_reduction
            ))
            .top_right_with_margins_on(state.ids.stats_alignment, 120.0, 130.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(16))
            .color(TEXT_COLOR)
            .set(state.ids.stats, ui);
        }
        // Bag Slots
        // Create available inventory slot widgets
        if state.ids.inv_slots.len() < inventory.len() {
            state.update(|s| {
                s.ids
                    .inv_slots
                    .resize(inventory.len(), &mut ui.widget_id_generator());
            });
        }
        // Display inventory contents
        let mut slot_maker = SlotMaker {
            empty_slot: self.imgs.inv_slot,
            filled_slot: self.imgs.inv_slot,
            selected_slot: self.imgs.inv_slot_sel,
            background_color: Some(UI_MAIN),
            content_size: ContentSize {
                width_height_ratio: 1.0,
                max_fraction: 0.75,
            },
            selected_content_scale: 1.067,
            amount_font: self.fonts.cyri.conrod_id,
            amount_margins: Vec2::new(-4.0, 0.0),
            amount_font_size: self.fonts.cyri.scale(12),
            amount_text_color: TEXT_COLOR,
            content_source: inventory,
            image_source: self.item_imgs,
            slot_manager: Some(self.slot_manager),
        };
        for (i, item) in inventory.slots().iter().enumerate() {
            let x = i % 9;
            let y = i / 9;

            // Slot
            let slot_widget = slot_maker
                .fabricate(InventorySlot(i), [40.0; 2])
                .top_left_with_margins_on(
                    state.ids.inv_alignment,
                    0.0 + y as f64 * (40.0),
                    0.0 + x as f64 * (40.0),
                );
            if let Some(item) = item {
                let (title, desc) = super::util::item_text(item);
                let quality_col = get_quality_col(item);
                let quality_col_img = match item.quality() {
                    Quality::Low => self.imgs.inv_slot_grey,
                    Quality::Common => self.imgs.inv_slot,
                    Quality::Moderate => self.imgs.inv_slot_green,
                    Quality::High => self.imgs.inv_slot_blue,
                    Quality::Epic => self.imgs.inv_slot_purple,
                    Quality::Legendary => self.imgs.inv_slot_gold,
                    Quality::Artifact => self.imgs.inv_slot_orange,
                    _ => self.imgs.inv_slot_red,
                };
                slot_widget
                    .filled_slot(quality_col_img)
                    .with_tooltip(
                        self.tooltip_manager,
                        title,
                        &*desc,
                        &item_tooltip,
                        quality_col,
                    )
                    .set(state.ids.inv_slots[i], ui);
            } else {
                slot_widget.set(state.ids.inv_slots[i], ui);
            }
        }

        // Stats Button
        if Button::image(self.imgs.button)
            .w_h(92.0, 22.0)
            .mid_top_with_margin_on(state.ids.bg, 435.0)
            .hover_image(self.imgs.button_hover)
            .press_image(self.imgs.button_press)
            .label(if self.show.stats {
                &self.localized_strings.get("hud.bag.armor")
            } else {
                &self.localized_strings.get("hud.bag.stats")
            })
            .label_y(conrod_core::position::Relative::Scalar(1.0))
            .label_color(TEXT_COLOR)
            .label_font_size(self.fonts.cyri.scale(12))
            .label_font_id(self.fonts.cyri.conrod_id)
            .set(state.ids.stats_button, ui)
            .was_clicked()
        {
            return Some(Event::Stats);
        };
        // Tabs
        if Button::image(self.imgs.inv_tab_active)
            .w_h(28.0, 44.0)
            .bottom_left_with_margins_on(state.ids.bg, 172.0, 13.0)
            .image_color(UI_MAIN)
            .set(state.ids.tab_1, ui)
            .was_clicked()
        {}
        if Button::image(self.imgs.inv_tab_inactive)
            .w_h(28.0, 44.0)
            .hover_image(self.imgs.inv_tab_inactive_hover)
            .press_image(self.imgs.inv_tab_inactive_press)
            .image_color(UI_HIGHLIGHT_0)
            .down_from(state.ids.tab_1, 0.0)
            .with_tooltip(
                self.tooltip_manager,
                "Not yet Available",
                "",
                &item_tooltip,
                TEXT_COLOR,
            )
            .set(state.ids.tab_2, ui)
            .was_clicked()
        {}
        if Button::image(self.imgs.inv_tab_inactive)
            .w_h(28.0, 44.0)
            .hover_image(self.imgs.inv_tab_inactive_hover)
            .press_image(self.imgs.inv_tab_inactive_press)
            .down_from(state.ids.tab_2, 0.0)
            .image_color(UI_HIGHLIGHT_0)
            .with_tooltip(
                self.tooltip_manager,
                "Not yet Available",
                "",
                &item_tooltip,
                TEXT_COLOR,
            )
            .set(state.ids.tab_3, ui)
            .was_clicked()
        {}
        if Button::image(self.imgs.inv_tab_inactive)
            .w_h(28.0, 44.0)
            .hover_image(self.imgs.inv_tab_inactive_hover)
            .press_image(self.imgs.inv_tab_inactive_press)
            .down_from(state.ids.tab_3, 0.0)
            .image_color(UI_HIGHLIGHT_0)
            .with_tooltip(
                self.tooltip_manager,
                "Not yet Available",
                "",
                &item_tooltip,
                TEXT_COLOR,
            )
            .set(state.ids.tab_4, ui)
            .was_clicked()
        {}
        // Close button
        if Button::image(self.imgs.close_btn)
            .w_h(24.0, 25.0)
            .hover_image(self.imgs.close_btn_hover)
            .press_image(self.imgs.close_btn_press)
            .top_right_with_margins_on(state.ids.bg, 0.0, 0.0)
            .set(state.ids.bag_close, ui)
            .was_clicked()
        {
            event = Some(Event::Close);
        }
        event
    }
}
