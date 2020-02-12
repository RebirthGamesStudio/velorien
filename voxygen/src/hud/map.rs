use super::{
    img_ids::{Imgs, ImgsRot},
    Fonts, Show, TEXT_COLOR,
};
use crate::ui::img_ids;
use client::{self, Client};
use common::{comp, terrain::TerrainChunkSize, vol::RectVolSize};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Color, Colorable, Positionable, Sizeable, Widget, WidgetCommon,
};
use specs::WorldExt;
use vek::*;
widget_ids! {
    struct Ids {
        map_frame,
        map_bg,
        map_icon,
        map_close,
        map_title,
        map_frame_l,
        map_frame_r,
        map_frame_bl,
        map_frame_br,
        location_name,
        indicator,
        grid,
    }
}

#[derive(WidgetCommon)]
pub struct Map<'a> {
    _show: &'a Show,
    client: &'a Client,
    world_map: &'a (img_ids::Rotations, Vec2<u32>),
    imgs: &'a Imgs,
    rot_imgs: &'a ImgsRot,
    fonts: &'a Fonts,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    _pulse: f32,
    velocity: f32,
}
impl<'a> Map<'a> {
    pub fn new(
        show: &'a Show,
        client: &'a Client,
        imgs: &'a Imgs,
        rot_imgs: &'a ImgsRot,
        world_map: &'a (img_ids::Rotations, Vec2<u32>),
        fonts: &'a Fonts,
        pulse: f32,
        velocity: f32,
    ) -> Self {
        Self {
            _show: show,
            imgs,
            rot_imgs,
            world_map,
            client,
            fonts,
            common: widget::CommonBuilder::default(),
            _pulse: pulse,
            velocity,
        }
    }
}

pub struct State {
    ids: Ids,
}

pub enum Event {
    Close,
}

impl<'a> Widget for Map<'a> {
    type Event = Option<Event>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style { () }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, .. } = args;
        // Set map transparency to 0.5 when player is moving
        let mut fade = 1.0;
        if self.velocity > 2.5 {
            fade = 0.7
        };

        // BG
        Rectangle::fill_with([824.0, 976.0], color::TRANSPARENT)
            .mid_top_with_margin_on(ui.window, 15.0)
            .scroll_kids()
            .scroll_kids_vertically()
            .set(state.ids.map_bg, ui);

        // Frame
        Image::new(self.imgs.map_frame_l)
            .top_left_with_margins_on(state.ids.map_bg, 0.0, 0.0)
            .w_h(412.0, 488.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade)))
            .set(state.ids.map_frame_l, ui);
        Image::new(self.imgs.map_frame_r)
            .right_from(state.ids.map_frame_l, 0.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade)))
            .w_h(412.0, 488.0)
            .set(state.ids.map_frame_r, ui);
        Image::new(self.imgs.map_frame_br)
            .down_from(state.ids.map_frame_r, 0.0)
            .w_h(412.0, 488.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade)))
            .set(state.ids.map_frame_br, ui);
        Image::new(self.imgs.map_frame_bl)
            .down_from(state.ids.map_frame_l, 0.0)
            .w_h(412.0, 488.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade)))
            .set(state.ids.map_frame_bl, ui);

        // Icon
        Image::new(self.imgs.map_icon)
            .w_h(224.0 / 3.0, 224.0 / 3.0)
            .top_left_with_margins_on(state.ids.map_frame, -10.0, -10.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade)))
            .set(state.ids.map_icon, ui);

        // X-Button
        if Button::image(self.imgs.close_button)
            .w_h(28.0, 28.0)
            .hover_image(self.imgs.close_button_hover)
            .press_image(self.imgs.close_button_press)
            .color(Color::Rgba(1.0, 1.0, 1.0, fade - 0.5))
            .top_right_with_margins_on(state.ids.map_frame_r, 0.0, 0.0)
            .set(state.ids.map_close, ui)
            .was_clicked()
        {
            return Some(Event::Close);
        }

        // Location Name
        match self.client.current_chunk() {
            Some(chunk) => Text::new(chunk.meta().name())
                .mid_top_with_margin_on(state.ids.map_bg, 55.0)
                .font_size(60)
                .color(TEXT_COLOR)
                .font_id(self.fonts.alkhemi)
                .parent(state.ids.map_frame_r)
                .set(state.ids.location_name, ui),
            None => Text::new(" ")
                .mid_top_with_margin_on(state.ids.map_bg, 3.0)
                .font_size(40)
                .font_id(self.fonts.alkhemi)
                .color(TEXT_COLOR)
                .set(state.ids.location_name, ui),
        }

        // Map Image
        let (world_map, worldsize) = self.world_map;
        let worldsize = worldsize.map2(TerrainChunkSize::RECT_SIZE, |e, f| e as f64 * f as f64);

        Image::new(world_map.none)
            .middle_of(state.ids.map_bg)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, fade + 0.5)))
            .w_h(700.0, 700.0)
            .parent(state.ids.map_bg)
            .set(state.ids.grid, ui);
        // Coordinates
        let player_pos = self
            .client
            .state()
            .ecs()
            .read_storage::<comp::Pos>()
            .get(self.client.entity())
            .map_or(Vec3::zero(), |pos| pos.0);

        let x = player_pos.x as f64 / worldsize.x * 700.0;
        let y = player_pos.y as f64 / worldsize.y * 700.0;
        let indic_scale = 0.6;
        Image::new(self.rot_imgs.indicator_mmap_small.target_north)
            .bottom_left_with_margins_on(
                state.ids.grid,
                y - 37.0 * indic_scale / 2.0,
                x - 32.0 * indic_scale / 2.0,
            )
            .w_h(32.0 * indic_scale, 37.0 * indic_scale)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, 1.0)))
            .floating(true)
            .parent(ui.window)
            .set(state.ids.indicator, ui);

        None
    }
}
