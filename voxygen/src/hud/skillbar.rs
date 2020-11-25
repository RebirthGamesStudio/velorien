use super::{
    hotbar,
    img_ids::{Imgs, ImgsRot},
    item_imgs::ItemImgs,
    slots, BarNumbers, ShortcutNumbers, Show, BLACK, CRITICAL_HP_COLOR, LOW_HP_COLOR,
    STAMINA_COLOR, TEXT_COLOR, UI_HIGHLIGHT_0, UI_MAIN, XP_COLOR,
};
use crate::{
    i18n::Localization,
    ui::{
        fonts::Fonts,
        slot::{ContentSize, SlotMaker},
        ImageFrame, Tooltip, TooltipManager, Tooltipable,
    },
    window::GameInput,
    GlobalState,
};
use common::comp::{
    item::{
        tool::{Tool, ToolKind},
        Hands, ItemKind,
    },
    Energy, Health, Inventory, Loadout, Stats,
};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Color, Colorable, Positionable, Sizeable, Widget, WidgetCommon,
};
use std::time::{Duration, Instant};
use vek::*;

widget_ids! {
    struct Ids {
        // Death message
        death_message_1,
        death_message_2,
        death_message_1_bg,
        death_message_2_bg,
        death_bg,
        // Level up message
        level_up,
        level_down,
        level_align,
        level_message,
        level_message_bg,
        // Hurt BG
        hurt_bg,
        // Skillbar
        alignment,
        bg,
        frame,
        m1_ico,
        m2_ico,
        // Level
        level_bg,
        level,
        // Exp-Bar
        exp_alignment,
        exp_filling,
        // HP-Bar
        hp_alignment,
        hp_filling,
        hp_txt_alignment,
        hp_txt_bg,
        hp_txt,
        // Stamina-Bar
        stamina_alignment,
        stamina_filling,
        stamina_txt_alignment,
        stamina_txt_bg,
        stamina_txt,
        // Slots
        m1_slot,
        m1_slot_bg,
        m1_text,
        m1_text_bg,
        m1_slot_act,
        m1_content,
        m2_slot,
        m2_slot_bg,
        m2_text,
        m2_text_bg,
        m2_slot_act,
        m2_content,
        slot1,
        slot1_text,
        slot1_text_bg,
        slot2,
        slot2_text,
        slot2_text_bg,
        slot3,
        slot3_text,
        slot3_text_bg,
        slot4,
        slot4_text,
        slot4_text_bg,
        slot5,
        slot5_text,
        slot5_text_bg,
        slot6,
        slot6_text,
        slot6_text_bg,
        slot7,
        slot7_text,
        slot7_text_bg,
        slot8,
        slot8_text,
        slot8_text_bg,
        slot9,
        slot9_text,
        slot9_text_bg,
        slot10,
        slot10_text,
        slot10_text_bg,
    }
}

#[derive(WidgetCommon)]
pub struct Skillbar<'a> {
    global_state: &'a GlobalState,
    imgs: &'a Imgs,
    item_imgs: &'a ItemImgs,
    fonts: &'a Fonts,
    rot_imgs: &'a ImgsRot,
    stats: &'a Stats,
    health: &'a Health,
    loadout: &'a Loadout,
    energy: &'a Energy,
    // character_state: &'a CharacterState,
    // controller: &'a ControllerInputs,
    inventory: &'a Inventory,
    hotbar: &'a hotbar::State,
    tooltip_manager: &'a mut TooltipManager,
    slot_manager: &'a mut slots::SlotManager,
    localized_strings: &'a Localization,
    pulse: f32,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    show: &'a Show,
}

impl<'a> Skillbar<'a> {
    #[allow(clippy::too_many_arguments)] // TODO: Pending review in #587
    pub fn new(
        global_state: &'a GlobalState,
        imgs: &'a Imgs,
        item_imgs: &'a ItemImgs,
        fonts: &'a Fonts,
        rot_imgs: &'a ImgsRot,
        stats: &'a Stats,
        health: &'a Health,
        loadout: &'a Loadout,
        energy: &'a Energy,
        // character_state: &'a CharacterState,
        pulse: f32,
        // controller: &'a ControllerInputs,
        inventory: &'a Inventory,
        hotbar: &'a hotbar::State,
        tooltip_manager: &'a mut TooltipManager,
        slot_manager: &'a mut slots::SlotManager,
        localized_strings: &'a Localization,
        show: &'a Show,
    ) -> Self {
        Self {
            global_state,
            imgs,
            item_imgs,
            fonts,
            rot_imgs,
            stats,
            health,
            loadout,
            energy,
            common: widget::CommonBuilder::default(),
            // character_state,
            pulse,
            // controller,
            inventory,
            hotbar,
            tooltip_manager,
            slot_manager,
            localized_strings,
            show,
        }
    }
}

pub struct State {
    ids: Ids,
    last_level: u32,
    last_update_level: Instant,
}

impl<'a> Widget for Skillbar<'a> {
    type Event = ();
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            last_level: 1,
            last_update_level: Instant::now(),
        }
    }

    #[allow(clippy::unused_unit)] // TODO: Pending review in #587
    fn style(&self) -> Self::Style { () }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, .. } = args;

        let level = if self.stats.level.level() > 999 {
            "A".to_string()
        } else {
            (self.stats.level.level()).to_string()
        };

        let exp_percentage = (self.stats.exp.current() as f64) / (self.stats.exp.maximum() as f64);

        let mut hp_percentage = self.health.current() as f64 / self.health.maximum() as f64 * 100.0;
        let mut energy_percentage =
            self.energy.current() as f64 / self.energy.maximum() as f64 * 100.0;
        if self.health.is_dead {
            hp_percentage = 0.0;
            energy_percentage = 0.0;
        };

        let bar_values = self.global_state.settings.gameplay.bar_numbers;
        let shortcuts = self.global_state.settings.gameplay.shortcut_numbers;

        let hp_ani = (self.pulse * 4.0/* speed factor */).cos() * 0.5 + 0.8; //Animation timer
        let crit_hp_color: Color = Color::Rgba(0.79, 0.19, 0.17, hp_ani);

        let localized_strings = self.localized_strings;

        // Level Up Message
        if !self.show.intro {
            let current_level = self.stats.level.level();
            const FADE_IN_LVL: f32 = 1.0;
            const FADE_HOLD_LVL: f32 = 3.0;
            const FADE_OUT_LVL: f32 = 2.0;
            // Fade
            // Check if no other popup is displayed and a new one is needed
            if state.last_update_level.elapsed()
                > Duration::from_secs_f32(FADE_IN_LVL + FADE_HOLD_LVL + FADE_OUT_LVL)
                && state.last_level != current_level
            {
                // Update last_value
                state.update(|s| s.last_level = current_level);
                state.update(|s| s.last_update_level = Instant::now());
            };

            let seconds_level = state.last_update_level.elapsed().as_secs_f32();
            let fade_level = if current_level == 1 {
                0.0
            } else if seconds_level < FADE_IN_LVL {
                seconds_level / FADE_IN_LVL
            } else if seconds_level < FADE_IN_LVL + FADE_HOLD_LVL {
                1.0
            } else {
                (1.0 - (seconds_level - FADE_IN_LVL - FADE_HOLD_LVL) / FADE_OUT_LVL).max(0.0)
            };
            // Contents
            Rectangle::fill_with([82.0 * 4.0, 40.0 * 4.0], color::TRANSPARENT)
                .mid_top_with_margin_on(ui.window, 300.0)
                .set(state.ids.level_align, ui);
            let level_up_text = &localized_strings
                .get("char_selection.level_fmt")
                .replace("{level_nb}", &self.stats.level.level().to_string());
            Text::new(&level_up_text)
                .middle_of(state.ids.level_align)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, fade_level))
                .set(state.ids.level_message_bg, ui);
            Text::new(&level_up_text)
                .bottom_left_with_margins_on(state.ids.level_message_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(1.0, 1.0, 1.0, fade_level))
                .set(state.ids.level_message, ui);
            Image::new(self.imgs.level_up)
                .w_h(82.0 * 4.0, 9.0 * 4.0)
                .mid_top_with_margin_on(state.ids.level_align, 0.0)
                .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade_level)))
                .graphics_for(state.ids.level_align)
                .set(state.ids.level_up, ui);
            Image::new(self.imgs.level_down)
                .w_h(82.0 * 4.0, 9.0 * 4.0)
                .mid_bottom_with_margin_on(state.ids.level_align, 0.0)
                .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade_level)))
                .graphics_for(state.ids.level_align)
                .set(state.ids.level_down, ui);
        }
        // Death message
        if self.health.is_dead {
            if let Some(key) = self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Respawn)
            {
                Text::new(&localized_strings.get("hud.you_died"))
                    .middle_of(ui.window)
                    .font_size(self.fonts.cyri.scale(50))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                    .set(state.ids.death_message_1_bg, ui);
                Text::new(
                    &localized_strings
                        .get("hud.press_key_to_respawn")
                        .replace("{key}", key.to_string().as_str()),
                )
                .mid_bottom_with_margin_on(state.ids.death_message_1_bg, -120.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_2_bg, ui);
                Text::new(&localized_strings.get("hud.you_died"))
                    .bottom_left_with_margins_on(state.ids.death_message_1_bg, 2.0, 2.0)
                    .font_size(self.fonts.cyri.scale(50))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(CRITICAL_HP_COLOR)
                    .set(state.ids.death_message_1, ui);
                Text::new(
                    &localized_strings
                        .get("hud.press_key_to_respawn")
                        .replace("{key}", key.to_string().as_str()),
                )
                .bottom_left_with_margins_on(state.ids.death_message_2_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(CRITICAL_HP_COLOR)
                .set(state.ids.death_message_2, ui);
            }
        }
        // Skillbar
        // Alignment and BG
        Rectangle::fill_with([524.0, 80.0], color::TRANSPARENT)
            .mid_bottom_with_margin_on(ui.window, 10.0)
            .set(state.ids.alignment, ui);
        Image::new(self.imgs.skillbar_bg)
            .w_h(480.0, 80.0)
            .color(Some(UI_MAIN))
            .middle_of(state.ids.alignment)
            .set(state.ids.bg, ui);
        // Level
        let lvl_size = match self.stats.level.level() {
            11..=99 => 13,
            100..=999 => 10,
            _ => 14,
        };
        Text::new(&level)
            .mid_top_with_margin_on(state.ids.bg, 3.0)
            .font_size(self.fonts.cyri.scale(lvl_size))
            .font_id(self.fonts.cyri.conrod_id)
            .color(TEXT_COLOR)
            .set(state.ids.level, ui);
        // Exp-Bar
        Rectangle::fill_with([476.0, 8.0], color::TRANSPARENT)
            .mid_bottom_with_margin_on(state.ids.bg, 4.0)
            .set(state.ids.exp_alignment, ui);
        Image::new(self.imgs.bar_content)
            .w_h(476.0 * exp_percentage, 8.0)
            .color(Some(XP_COLOR))
            .bottom_left_with_margins_on(state.ids.exp_alignment, 0.0, 0.0)
            .set(state.ids.exp_filling, ui);
        // Health and Stamina bar
        // Alignment
        Rectangle::fill_with([240.0, 17.0], color::TRANSPARENT)
            .top_left_with_margins_on(state.ids.alignment, 0.0, 0.0)
            .set(state.ids.hp_alignment, ui);
        Rectangle::fill_with([240.0, 17.0], color::TRANSPARENT)
            .top_right_with_margins_on(state.ids.alignment, 0.0, 0.0)
            .set(state.ids.stamina_alignment, ui);
        let health_col = match hp_percentage as u8 {
            0..=20 => crit_hp_color,
            21..=40 => LOW_HP_COLOR,
            _ => self
                .global_state
                .settings
                .accessibility
                .hud_colors
                .health_color(),
        };
        // Content
        Image::new(self.imgs.bar_content)
            .w_h(216.0 * hp_percentage / 100.0, 14.0)
            .color(Some(health_col))
            .top_right_with_margins_on(state.ids.hp_alignment, 4.0, 0.0)
            .set(state.ids.hp_filling, ui);
        Image::new(self.imgs.bar_content)
            .w_h(216.0 * energy_percentage / 100.0, 14.0)
            .color(Some(STAMINA_COLOR))
            .top_left_with_margins_on(state.ids.stamina_alignment, 4.0, 0.0)
            .set(state.ids.stamina_filling, ui);
        Rectangle::fill_with([219.0, 14.0], color::TRANSPARENT)
            .top_left_with_margins_on(state.ids.hp_alignment, 4.0, 20.0)
            .set(state.ids.hp_txt_alignment, ui);
        Rectangle::fill_with([219.0, 14.0], color::TRANSPARENT)
            .top_right_with_margins_on(state.ids.stamina_alignment, 4.0, 20.0)
            .set(state.ids.stamina_txt_alignment, ui);
        // Bar Text
        // Values
        if let BarNumbers::Values = bar_values {
            let mut hp_txt = format!(
                "{}/{}",
                (self.health.current() / 10).max(1) as u32, /* Don't show 0 health for
                                                             * living players */
                (self.health.maximum() / 10) as u32
            );
            let mut energy_txt = format!("{}", energy_percentage as u32);
            if self.health.is_dead {
                hp_txt = self.localized_strings.get("hud.group.dead").to_string();
                energy_txt = self.localized_strings.get("hud.group.dead").to_string();
            };
            Text::new(&hp_txt)
                .middle_of(state.ids.hp_txt_alignment)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.hp_txt_bg, ui);
            Text::new(&hp_txt)
                .bottom_left_with_margins_on(state.ids.hp_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.hp_txt, ui);
            Text::new(&energy_txt)
                .middle_of(state.ids.stamina_txt_alignment)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.stamina_txt_bg, ui);
            Text::new(&energy_txt)
                .bottom_left_with_margins_on(state.ids.stamina_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.stamina_txt, ui);
        }
        //Percentages
        if let BarNumbers::Percent = bar_values {
            let mut hp_txt = format!("{}%", hp_percentage as u32);
            let mut energy_txt = format!("{}", energy_percentage as u32);
            if self.health.is_dead {
                hp_txt = self.localized_strings.get("hud.group.dead").to_string();
                energy_txt = self.localized_strings.get("hud.group.dead").to_string();
            };
            Text::new(&hp_txt)
                .middle_of(state.ids.hp_txt_alignment)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.hp_txt_bg, ui);
            Text::new(&hp_txt)
                .bottom_left_with_margins_on(state.ids.hp_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.hp_txt, ui);
            Text::new(&energy_txt)
                .middle_of(state.ids.stamina_txt_alignment)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.stamina_txt_bg, ui);
            Text::new(&energy_txt)
                .bottom_left_with_margins_on(state.ids.stamina_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.stamina_txt, ui);
        }
        // Slots
        let content_source = (self.hotbar, self.inventory, self.loadout, self.energy); // TODO: avoid this
        let image_source = (self.item_imgs, self.imgs);
        let mut slot_maker = SlotMaker {
            // TODO: is a separate image needed for the frame?
            empty_slot: self.imgs.inv_slot,
            filled_slot: self.imgs.inv_slot,
            selected_slot: self.imgs.inv_slot_sel,
            background_color: None,
            content_size: ContentSize {
                width_height_ratio: 1.0,
                max_fraction: 0.8, /* Changes the item image size by setting a maximum fraction
                                    * of either the width or height */
            },
            selected_content_scale: 1.0,
            amount_font: self.fonts.cyri.conrod_id,
            amount_margins: Vec2::new(1.0, 1.0),
            amount_font_size: self.fonts.cyri.scale(12),
            amount_text_color: TEXT_COLOR,
            content_source: &content_source,
            image_source: &image_source,
            slot_manager: Some(self.slot_manager),
        };
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
        // Helper
        let tooltip_text = |slot| {
            content_source
                .0
                .get(slot)
                .and_then(|content| match content {
                    hotbar::SlotContents::Inventory(i) => content_source
                        .1
                        .get(i)
                        .map(|item| (item.name(), item.description())),
                    hotbar::SlotContents::Ability3 => content_source
                        .2
                        .active_item
                        .as_ref()
                        .map(|i| i.item.kind())
                        .and_then(|kind| match kind {
                            ItemKind::Tool(Tool { kind, .. }) => match kind {
                                ToolKind::Hammer => Some((
                                    "Smash of Doom",
                                    "\nAn AOE attack with knockback. \nLeaps to position of \
                                     cursor.",
                                )),
                                ToolKind::Axe => {
                                    Some(("Spin Leap", "\nA slashing running spin leap."))
                                },
                                ToolKind::Staff => Some((
                                    "Firebomb",
                                    "\nWhirls a big fireball into the air. \nExplodes the ground \
                                     and does\na big amount of damage",
                                )),
                                ToolKind::Sword => Some((
                                    "Whirlwind",
                                    "\nMove forward while spinning with \n your sword.",
                                )),
                                ToolKind::Bow => Some((
                                    "Burst",
                                    "\nLaunches a burst of arrows at the top \nof a running leap.",
                                )),
                                ToolKind::Debug => Some((
                                    "Possessing Arrow",
                                    "\nShoots a poisonous arrow.\nLets you control your target.",
                                )),
                                _ => None,
                            },
                            _ => None,
                        }),
                })
        };
        // Slot 1-5
        // Slot 1
        slot_maker.empty_slot = self.imgs.inv_slot;
        slot_maker.selected_slot = self.imgs.inv_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::One, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .bottom_left_with_margins_on(state.ids.frame, 15.0, 22.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::One) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot1, ui);
        } else {
            slot.set(state.ids.slot1, ui);
        }
        // Slot 2
        let slot = slot_maker
            .fabricate(hotbar::Slot::Two, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot1, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Two) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot2, ui);
        } else {
            slot.set(state.ids.slot2, ui);
        }
        // Slot 3
        let slot = slot_maker
            .fabricate(hotbar::Slot::Three, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot2, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Three) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot3, ui);
        } else {
            slot.set(state.ids.slot3, ui);
        }
        // Slot 4
        let slot = slot_maker
            .fabricate(hotbar::Slot::Four, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot3, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Three) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot4, ui);
        } else {
            slot.set(state.ids.slot4, ui);
        }
        // Slot 5
        let slot = slot_maker
            .fabricate(hotbar::Slot::Five, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot4, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Three) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot5, ui);
        } else {
            slot.set(state.ids.slot5, ui);
        }
        // Slot M1
        Image::new(self.imgs.inv_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.slot5, 0.0)
            .set(state.ids.m1_slot_bg, ui);
        Button::image(
            match self.loadout.active_item.as_ref().map(|i| i.item.kind()) {
                Some(ItemKind::Tool(Tool { kind, .. })) => match kind {
                    ToolKind::Sword => self.imgs.twohsword_m1,
                    ToolKind::Dagger => self.imgs.onehdagger_m1,
                    ToolKind::Shield => self.imgs.onehshield_m1,
                    ToolKind::Hammer => self.imgs.twohhammer_m1,
                    ToolKind::Axe => self.imgs.twohaxe_m1,
                    ToolKind::Bow => self.imgs.bow_m1,
                    ToolKind::Sceptre => self.imgs.heal_0,
                    ToolKind::Staff => self.imgs.fireball,
                    ToolKind::Debug => self.imgs.flyingrod_m1,
                    _ => self.imgs.nothing,
                },
                _ => self.imgs.nothing,
            },
        ) // Insert Icon here
        .w_h(36.0, 36.0)
        .middle_of(state.ids.m1_slot_bg)
        .set(state.ids.m1_content, ui);
        // Slot M2
        Image::new(self.imgs.inv_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.m1_slot_bg, 0.0)
            .set(state.ids.m2_slot, ui);

        let active_tool_kind = match self.loadout.active_item.as_ref().map(|i| i.item.kind()) {
            Some(ItemKind::Tool(Tool { kind, .. })) => Some(kind),
            _ => None,
        };

        let second_tool_kind = match self.loadout.second_item.as_ref().map(|i| i.item.kind()) {
            Some(ItemKind::Tool(Tool { kind, .. })) => Some(kind),
            _ => None,
        };

        let tool_kind = match (
            active_tool_kind.map(|tk| tk.hands()),
            second_tool_kind.map(|tk| tk.hands()),
        ) {
            (Some(Hands::TwoHand), _) => active_tool_kind,
            (_, Some(Hands::OneHand)) => second_tool_kind,
            (_, _) => None,
        };

        Image::new(self.imgs.inv_slot)
            .w_h(40.0, 40.0)
            .middle_of(state.ids.m2_slot)
            .set(state.ids.m2_slot_bg, ui);
        Button::image(match tool_kind {
            Some(ToolKind::Sword) => self.imgs.twohsword_m2,
            Some(ToolKind::Dagger) => self.imgs.onehdagger_m2,
            Some(ToolKind::Shield) => self.imgs.onehshield_m2,
            Some(ToolKind::Hammer) => self.imgs.hammergolf,
            Some(ToolKind::Axe) => self.imgs.axespin,
            Some(ToolKind::Bow) => self.imgs.bow_m2,
            Some(ToolKind::Sceptre) => self.imgs.heal_bomb,
            Some(ToolKind::Staff) => self.imgs.flamethrower,
            Some(ToolKind::Debug) => self.imgs.flyingrod_m2,
            _ => self.imgs.nothing,
        })
        .w_h(36.0, 36.0)
        .middle_of(state.ids.m2_slot_bg)
        .image_color(match tool_kind {
            // TODO Automate this to grey out unavailable M2 skills
            Some(ToolKind::Sword) => {
                if self.energy.current() as f64 >= 200.0 {
                    Color::Rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::Rgba(0.3, 0.3, 0.3, 0.8)
                }
            },
            Some(ToolKind::Sceptre) => {
                if self.energy.current() as f64 >= 400.0 {
                    Color::Rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::Rgba(0.3, 0.3, 0.3, 0.8)
                }
            },
            Some(ToolKind::Axe) => {
                if self.energy.current() as f64 >= 100.0 {
                    Color::Rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::Rgba(0.3, 0.3, 0.3, 0.8)
                }
            },
            _ => Color::Rgba(1.0, 1.0, 1.0, 1.0),
        })
        .set(state.ids.m2_content, ui);
        // Slot 6-10
        // Slot 6
        slot_maker.empty_slot = self.imgs.inv_slot;
        slot_maker.selected_slot = self.imgs.inv_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::Six, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.m2_slot_bg, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Six) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot6, ui);
        } else {
            slot.set(state.ids.slot6, ui);
        }
        // Slot 7
        let slot = slot_maker
            .fabricate(hotbar::Slot::Seven, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot6, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Seven) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot7, ui);
        } else {
            slot.set(state.ids.slot7, ui);
        }
        // Slot 8
        let slot = slot_maker
            .fabricate(hotbar::Slot::Eight, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot7, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Eight) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot8, ui);
        } else {
            slot.set(state.ids.slot8, ui);
        }
        // Slot 9
        let slot = slot_maker
            .fabricate(hotbar::Slot::Nine, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot8, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Nine) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot9, ui);
        } else {
            slot.set(state.ids.slot9, ui);
        }
        // Quickslot
        slot_maker.empty_slot = self.imgs.inv_slot;
        slot_maker.selected_slot = self.imgs.inv_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::Ten, [40.0; 2])
            .filled_slot(self.imgs.inv_slot)
            .right_from(state.ids.slot9, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Ten) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot10, ui);
        } else {
            slot.set(state.ids.slot10, ui);
        }

        // Shortcuts
        if let ShortcutNumbers::On = shortcuts {
            if let Some(slot1) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot1)
            {
                Text::new(slot1.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot1, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot1_text_bg, ui);
                Text::new(slot1.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot1_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot1_text, ui);
            }
            if let Some(slot2) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot2)
            {
                Text::new(slot2.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot2, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot2_text_bg, ui);
                Text::new(slot2.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot2_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot2_text, ui);
            }
            if let Some(slot3) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot3)
            {
                Text::new(slot3.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot3, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot3_text_bg, ui);
                Text::new(slot3.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot3_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot3_text, ui);
            }
            if let Some(slot4) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot4)
            {
                Text::new(slot4.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot4, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot4_text_bg, ui);
                Text::new(slot4.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot4_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot4_text, ui);
            }
            if let Some(slot5) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot5)
            {
                Text::new(slot5.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot5, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot5_text_bg, ui);
                Text::new(slot5.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot5_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot5_text, ui);
            }
            /*if let Some(m1) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Primary)
            {
                Text::new(m1.to_string().as_str())
                    .top_left_with_margins_on(state.ids.m1_slot, 5.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.m1_text_bg, ui);
                Text::new(m1.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.m1_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.m1_text, ui);
            }
            if let Some(m2) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Secondary)
            {
                Text::new(m2.to_string().as_str())
                    .top_right_with_margins_on(state.ids.m2_slot, 5.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.m2_text_bg, ui);
                Text::new(m2.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.m2_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.m2_text, ui);
            }*/
            if let Some(slot6) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot6)
            {
                Text::new(slot6.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot6, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot6_text_bg, ui);
                Text::new(slot6.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot6_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot6_text, ui);
            }
            if let Some(slot7) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot7)
            {
                Text::new(slot7.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot7, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot7_text_bg, ui);
                Text::new(slot7.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot7_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot7_text, ui);
            }
            if let Some(slot8) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot8)
            {
                Text::new(slot8.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot8, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot8_text_bg, ui);
                Text::new(slot8.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot8_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot8_text, ui);
            }
            if let Some(slot9) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot9)
            {
                Text::new(slot9.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot9, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot9_text_bg, ui);
                Text::new(slot9.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot9_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot9_text, ui);
            }
            if let Some(slot10) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot10)
            {
                Text::new(slot10.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot10, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot10_text_bg, ui);
                Text::new(slot10.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot10_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot10_text, ui);
            }
        };
        // Frame
        Image::new(self.imgs.skillbar_frame)
            .w_h(524.0, 80.0)
            .color(Some(UI_HIGHLIGHT_0))
            .middle_of(state.ids.bg)
            .floating(true)
            .set(state.ids.frame, ui);
        // M1 and M2 icons
        // TODO Don't show this if key bindings are changed
        Image::new(self.imgs.m1_ico)
            .w_h(16.0, 18.0)
            .mid_bottom_with_margin_on(state.ids.m1_content, -11.0)
            .set(state.ids.m1_ico, ui);
        Image::new(self.imgs.m2_ico)
            .w_h(16.0, 18.0)
            .mid_bottom_with_margin_on(state.ids.m2_content, -11.0)
            .set(state.ids.m2_ico, ui);
    }
}
