mod defaults;
mod primitive;
pub mod style;
mod widget;

pub use defaults::Defaults;

pub(self) use primitive::Primitive;

use super::{
    super::graphic::{self, Graphic, TexId},
    cache::Cache,
    widget::image,
    Font, FontId, RawFont, Rotation,
};
use crate::{
    render::{
        create_ui_quad, Consts, DynamicModel, Globals, Mesh, Renderer, UiLocals, UiMode, UiPipeline,
    },
    Error,
};
use common::util::srgba_to_linear;
use std::{convert::TryInto, ops::Range};
use vek::*;

enum DrawKind {
    Image(TexId),
    // Text and non-textured geometry
    Plain,
}
enum DrawCommand {
    Draw { kind: DrawKind, verts: Range<u32> },
    Scissor(Aabr<u16>),
    WorldPos(Option<usize>),
}
impl DrawCommand {
    fn image(verts: Range<usize>, id: TexId) -> DrawCommand {
        DrawCommand::Draw {
            kind: DrawKind::Image(id),
            // TODO: move conversion into helper method so we don't have to write it out so many
            // times
            verts: verts
                .start
                .try_into()
                .expect("Vertex count for UI rendering does not fit in a u32!")
                ..verts
                    .end
                    .try_into()
                    .expect("Vertex count for UI rendering does not fit in a u32!"),
        }
    }

    fn plain(verts: Range<usize>) -> DrawCommand {
        DrawCommand::Draw {
            kind: DrawKind::Plain,
            verts: verts
                .start
                .try_into()
                .expect("Vertex count for UI rendering does not fit in a u32!")
                ..verts
                    .end
                    .try_into()
                    .expect("Vertex count for UI rendering does not fit in a u32!"),
        }
    }
}

#[derive(PartialEq)]
enum State {
    Image(TexId),
    Plain,
}

// Optimization idea inspired by what I think iced wgpu renderer may be doing:
// Could have layers of things which don't intersect and thus can be reordered
// arbitrarily

pub struct IcedRenderer {
    //image_map: Map<(Image, Rotation)>,
    cache: Cache,
    // Model for drawing the ui
    model: DynamicModel<UiPipeline>,
    // Consts to specify positions of ingame elements (e.g. Nametags)
    ingame_locals: Vec<Consts<UiLocals>>,
    // Consts for default ui drawing position (ie the interface)
    interface_locals: Consts<UiLocals>,
    default_globals: Consts<Globals>,

    // Used to delay cache resizing until after current frame is drawn
    //need_cache_resize: bool,
    // Half of physical resolution
    half_res: Vec2<f32>,
    // Pixel perfection alignment
    align: Vec2<f32>,
    // Scale factor between physical and win dims
    p_scale: f32,
    // Pretend dims :) (i.e. scaled)
    win_dims: Vec2<f32>,
    // Scissor for the whole window
    window_scissor: Aabr<u16>,

    // Per-frame/update
    current_state: State,
    mesh: Mesh<UiPipeline>,
    glyphs: Vec<(usize, usize, Rgba<f32>, Vec2<u32>)>,
    // Output from glyph_brush in the previous frame
    // It can sometimes ask you to redraw with these instead (idk if that is done with
    // pre-positioned glyphs)
    last_glyph_verts: Vec<(Aabr<f32>, Aabr<f32>)>,
    start: usize,
    // Draw commands for the next render
    draw_commands: Vec<DrawCommand>,
    //current_scissor: Aabr<u16>,
}
impl IcedRenderer {
    pub fn new(
        renderer: &mut Renderer,
        scaled_dims: Vec2<f32>,
        default_font: Font,
    ) -> Result<Self, Error> {
        let (half_res, align, p_scale) =
            Self::calculate_resolution_dependents(renderer.get_resolution(), scaled_dims);

        Ok(Self {
            cache: Cache::new(renderer, default_font)?,
            draw_commands: Vec::new(),
            model: renderer.create_dynamic_model(100)?,
            interface_locals: renderer.create_consts(&[UiLocals::default()])?,
            default_globals: renderer.create_consts(&[Globals::default()])?,
            ingame_locals: Vec::new(),
            mesh: Mesh::new(),
            glyphs: Vec::new(),
            last_glyph_verts: Vec::new(),
            current_state: State::Plain,
            half_res,
            align,
            p_scale,
            win_dims: scaled_dims,
            window_scissor: default_scissor(renderer),
            start: 0,
            //current_scissor: default_scissor(renderer),
        })
    }

    pub fn add_font(&mut self, font: RawFont) -> FontId { self.cache.add_font(font) }

    pub fn add_graphic(&mut self, graphic: Graphic) -> graphic::Id {
        self.cache.add_graphic(graphic)
    }

    fn image_dims(&self, handle: image::Handle) -> (u32, u32) {
        self
            .cache
            .graphic_cache()
            .get_graphic_dims((handle, Rotation::None))
            // TODO: don't unwrap
            .unwrap()
    }

    pub fn resize(&mut self, scaled_dims: Vec2<f32>, renderer: &mut Renderer) {
        self.win_dims = scaled_dims;
        self.window_scissor = default_scissor(renderer);

        self.update_resolution_dependents(renderer.get_resolution());

        // Resize graphic cache
        self.cache.resize_graphic_cache(renderer);
        // Resize glyph cache
        self.cache.resize_glyph_cache(renderer).unwrap();
    }

    pub fn draw(&mut self, primitive: Primitive, renderer: &mut Renderer) {
        // Re-use memory
        self.draw_commands.clear();
        self.mesh.clear();
        self.glyphs.clear();

        self.current_state = State::Plain;
        self.start = 0;

        //self.current_scissor = default_scissor(renderer);

        self.draw_primitive(primitive, Vec2::zero(), renderer);

        // Enter the final command.
        self.draw_commands.push(match self.current_state {
            State::Plain => DrawCommand::plain(self.start..self.mesh.vertices().len()),
            State::Image(id) => DrawCommand::image(self.start..self.mesh.vertices().len(), id),
        });

        // Draw glyph cache (use for debugging).
        /*self.draw_commands
            .push(DrawCommand::Scissor(default_scissor(renderer)));
        self.start = self.mesh.vertices().len();
        self.mesh.push_quad(create_ui_quad(
            Aabr {
                min: (-1.0, -1.0).into(),
                max: (1.0, 1.0).into(),
            },
            Aabr {
                min: (0.0, 1.0).into(),
                max: (1.0, 0.0).into(),
            },
            Rgba::new(1.0, 1.0, 1.0, 0.3),
            UiMode::Text,
        ));
        self.draw_commands
            .push(DrawCommand::plain(self.start..self.mesh.vertices().len()));*/

        // Fill in placeholder glyph quads
        let (glyph_cache, cache_tex) = self.cache.glyph_cache_mut_and_tex();
        let half_res = self.half_res;

        let brush_result = glyph_cache.process_queued(
            |rect, tex_data| {
                let offset = [rect.min[0] as u16, rect.min[1] as u16];
                let size = [rect.width() as u16, rect.height() as u16];

                let new_data = tex_data
                    .iter()
                    .map(|x| [255, 255, 255, *x])
                    .collect::<Vec<[u8; 4]>>();

                if let Err(err) = renderer.update_texture(cache_tex, offset, size, &new_data) {
                    tracing::warn!("Failed to update glyph cache texture: {:?}", err);
                }
            },
            // Urgh more allocation we don't need
            |vertex_data| {
                let uv_rect = vertex_data.tex_coords;
                let uv = Aabr {
                    min: Vec2::new(uv_rect.min.x, uv_rect.max.y),
                    max: Vec2::new(uv_rect.max.x, uv_rect.min.y),
                };
                let pixel_coords = vertex_data.pixel_coords;
                let rect = Aabr {
                    min: Vec2::new(
                        pixel_coords.min.x as f32 / half_res.x - 1.0,
                        1.0 - pixel_coords.max.y as f32 / half_res.y,
                    ),
                    max: Vec2::new(
                        pixel_coords.max.x as f32 / half_res.x - 1.0,
                        1.0 - pixel_coords.min.y as f32 / half_res.y,
                    ),
                };
                (uv, rect)
            },
        );

        match brush_result {
            Ok(brush_action) => {
                match brush_action {
                    glyph_brush::BrushAction::Draw(verts) => self.last_glyph_verts = verts,
                    glyph_brush::BrushAction::ReDraw => {},
                }

                let glyphs = &self.glyphs;
                let mesh = &mut self.mesh;
                let p_scale = self.p_scale;
                let half_res = self.half_res;

                glyphs
                    .iter()
                    .flat_map(|(mesh_index, glyph_count, linear_color, offset)| {
                        let mesh_index = *mesh_index;
                        let linear_color = *linear_color;
                        // Could potentially pass this in as part of the extras
                        let offset = offset.map(|e| e as f32 * p_scale) / half_res;
                        (0..*glyph_count).map(move |i| (mesh_index + i * 6, linear_color, offset))
                    })
                    .zip(self.last_glyph_verts.iter())
                    .for_each(|((mesh_index, linear_color, offset), (uv, rect))| {
                        // TODO: add function to vek for this
                        let rect = Aabr {
                            min: rect.min + offset,
                            max: rect.max + offset,
                        };

                        mesh.replace_quad(
                            mesh_index,
                            create_ui_quad(rect, *uv, linear_color, UiMode::Text),
                        )
                    });
            },
            Err(glyph_brush::BrushError::TextureTooSmall { suggested: (x, y) }) => {
                tracing::error!(
                    "Texture to small for all glyphs, would need one of the size: ({}, {})",
                    x,
                    y
                );
            },
        }

        // Create a larger dynamic model if the mesh is larger than the current model
        // size.
        if self.model.vbuf.len() < self.mesh.vertices().len() {
            self.model = renderer
                .create_dynamic_model(self.mesh.vertices().len() * 4 / 3)
                .unwrap();
        }
        // Update model with new mesh.
        renderer.update_model(&self.model, &self.mesh, 0).unwrap();

        // Handle window resizing.
        /*if let Some(new_dims) = self.window_resized.take() {
            let (old_w, old_h) = self.scale.scaled_window_size().into_tuple();
            self.scale.window_resized(new_dims, renderer);
            let (w, h) = self.scale.scaled_window_size().into_tuple();
            self.ui.handle_event(Input::Resize(w, h));

            // Avoid panic in graphic cache when minimizing.
            // Avoid resetting cache if window size didn't change
            // Somewhat inefficient for elements that won't change size after a window resize
            let res = renderer.get_resolution();
            self.need_cache_resize = res.x > 0 && res.y > 0 && !(old_w == w && old_h == h);
        }*/
    }

    // Returns (half_res, align)
    fn calculate_resolution_dependents(
        res: Vec2<u16>,
        win_dims: Vec2<f32>,
    ) -> (Vec2<f32>, Vec2<f32>, f32) {
        let half_res = res.map(|e| e as f32 / 2.0);
        let align = align(res);
        // Assume to be the same in x and y for now...
        let p_scale = res.x as f32 / win_dims.x;

        (half_res, align, p_scale)
    }

    fn update_resolution_dependents(&mut self, res: Vec2<u16>) {
        let (half_res, align, p_scale) = Self::calculate_resolution_dependents(res, self.win_dims);
        self.half_res = half_res;
        self.align = align;
        self.p_scale = p_scale;
    }

    fn gl_aabr(&self, bounds: iced::Rectangle) -> Aabr<f32> {
        /*let (ui_win_w, ui_win_h) = self.win_dims.into_tuple();
        let (l, b) = aabr.min.into_tuple();
        let (r, t) = aabr.max.into_tuple();
        let vx = |x: f64| (x / ui_win_w * 2.0) as f32;
        let vy = |y: f64| (y / ui_win_h * 2.0) as f32;
        let min = Vec2::new(
            ((vx(l) * half_res.x + x_align).round() - x_align) / half_res.x,
            ((vy(b) * half_res.y + y_align).round() - y_align) / half_res.y,
        );
        let max = Vec2::new(
            ((vx(r) * half_res.x + x_align).round() - x_align) / half_res.x,
            ((vy(t) * half_res.y + y_align).round() - y_align) / half_res.y,
        );*/
        let flipped_y = self.win_dims.y - bounds.y;
        let half_win_dims = self.win_dims.map(|e| e / 2.0);
        let half_res = self.half_res;
        let min = (((Vec2::new(bounds.x, flipped_y - bounds.height) - half_win_dims)
            / half_win_dims
            * half_res
            + self.align)
            .map(|e| e.round())
            - self.align)
            / half_res;
        let max = (((Vec2::new(bounds.x + bounds.width, flipped_y) - half_win_dims)
            / half_win_dims
            * half_res
            + self.align)
            .map(|e| e.round())
            - self.align)
            / half_res;
        Aabr { min, max }
    }

    fn draw_primitive(&mut self, primitive: Primitive, offset: Vec2<u32>, renderer: &mut Renderer) {
        match primitive {
            Primitive::Group { primitives } => {
                primitives
                    .into_iter()
                    .for_each(|p| self.draw_primitive(p, offset, renderer));
            },
            Primitive::Image {
                handle,
                bounds,
                color,
            } => {
                let color = srgba_to_linear(color.map(|e| e as f32 / 255.0));
                // Don't draw a transparent image.
                if color[3] == 0.0 {
                    return;
                }

                let (graphic_id, rotation) = handle;
                let gl_aabr = self.gl_aabr(iced::Rectangle {
                    x: bounds.x + offset.x as f32,
                    y: bounds.y + offset.y as f32,
                    ..bounds
                });

                let graphic_cache = self.cache.graphic_cache_mut();

                match graphic_cache.get_graphic(graphic_id) {
                    Some(Graphic::Blank) | None => return,
                    _ => {},
                }

                let resolution = Vec2::new(
                    (gl_aabr.size().w * self.half_res.x).round() as u16,
                    (gl_aabr.size().h * self.half_res.y).round() as u16,
                );
                // Transform the source rectangle into uv coordinate.
                // TODO: Make sure this is right.
                let source_aabr = {
                    let (uv_l, uv_r, uv_b, uv_t) = (0.0, 1.0, 0.0, 1.0);
                    /*match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            ((l / image_w) as f32,
                            (r / image_w) as f32,
                            (b / image_h) as f32,
                            (t / image_h) as f32)
                        }
                        None => (0.0, 1.0, 0.0, 1.0),
                    };*/
                    Aabr {
                        min: Vec2::new(uv_l, uv_b),
                        max: Vec2::new(uv_r, uv_t),
                    }
                };

                // Cache graphic at particular resolution.
                let (uv_aabr, tex_id) = match graphic_cache.cache_res(
                    renderer,
                    graphic_id,
                    resolution,
                    source_aabr,
                    rotation,
                ) {
                    // TODO: get dims from graphic_cache (or have it return floats directly)
                    Some((aabr, tex_id)) => {
                        let cache_dims = graphic_cache
                            .get_tex(tex_id)
                            .get_dimensions()
                            .map(|e| e as f32);
                        let min = Vec2::new(aabr.min.x as f32, aabr.max.y as f32) / cache_dims;
                        let max = Vec2::new(aabr.max.x as f32, aabr.min.y as f32) / cache_dims;
                        (Aabr { min, max }, tex_id)
                    },
                    None => return,
                };

                // Switch to the image state if we are not in it already or if a different
                // texture id was being used.
                self.switch_state(State::Image(tex_id));

                self.mesh
                    .push_quad(create_ui_quad(gl_aabr, uv_aabr, color, UiMode::Image));
            },
            Primitive::Rectangle {
                bounds,
                linear_color,
            } => {
                // Don't draw a transparent rectangle.
                if linear_color[3] == 0.0 {
                    return;
                }

                self.switch_state(State::Plain);

                let gl_aabr = self.gl_aabr(iced::Rectangle {
                    x: bounds.x + offset.x as f32,
                    y: bounds.y + offset.y as f32,
                    ..bounds
                });

                self.mesh.push_quad(create_ui_quad(
                    gl_aabr,
                    Aabr {
                        min: Vec2::zero(),
                        max: Vec2::zero(),
                    },
                    linear_color,
                    UiMode::Geometry,
                ));
            },
            Primitive::Text {
                glyphs,
                bounds: _bounds, // iced::Rectangle
                linear_color,
                /*font,
                 *horizontal_alignment,
                 *vertical_alignment, */
            } => {
                self.switch_state(State::Plain);

                // TODO: makes sure we are not doing all this work for hidden text
                // e.g. in chat
                let glyph_cache = self.cache.glyph_cache_mut();

                // Count glyphs
                let glyph_count = glyphs.len();

                // Queue the glyphs to be cached.
                glyph_cache.queue_pre_positioned(
                    glyphs,
                    // TODO: glyph_brush should document that these need to be the same length
                    vec![(); glyph_count],
                    // Since we already passed in `bounds` to position the glyphs some of this
                    // seems redundant...
                    // Note: we can't actually use this because dropping glyphs messeses up the
                    // counting and there is not a method provided to drop out of bounds
                    // glyphs while positioning them
                    glyph_brush::ab_glyph::Rect {
                        min: glyph_brush::ab_glyph::point(
                            -10000.0, //bounds.x * self.p_scale,
                            -10000.0, //bounds.y * self.p_scale,
                        ),
                        max: glyph_brush::ab_glyph::point(
                            10000.0, //(bounds.x + bounds.width) * self.p_scale,
                            10000.0, //(bounds.y + bounds.height) * self.p_scale,
                        ),
                    },
                );

                // Leave ui and verts blank to fill in when processing cached glyphs
                let zero_aabr = Aabr {
                    min: Vec2::broadcast(0.0),
                    max: Vec2::broadcast(0.0),
                };
                self.glyphs.push((
                    self.mesh.vertices().len(),
                    glyph_count,
                    linear_color,
                    offset,
                ));
                for _ in 0..glyph_count {
                    // Push placeholder quad
                    // Note: moving to some sort of layering / z based system would be an
                    // alternative to this (and might help with reducing draw
                    // calls)
                    self.mesh.push_quad(create_ui_quad(
                        zero_aabr,
                        zero_aabr,
                        linear_color,
                        UiMode::Text,
                    ));
                }
            },
            Primitive::Clip {
                bounds,
                offset: clip_offset,
                content,
            } => {
                let new_scissor = {
                    // TODO: incorporate current offset for nested Clips
                    let intersection = Aabr {
                        min: Vec2 {
                            x: (bounds.x * self.p_scale) as u16,
                            y: (bounds.y * self.p_scale) as u16,
                        },
                        max: Vec2 {
                            x: ((bounds.x + bounds.width) * self.p_scale) as u16,
                            y: ((bounds.y + bounds.height) * self.p_scale) as u16,
                        },
                    }
                    .intersection(self.window_scissor);

                    if intersection.is_valid() {
                        intersection
                    } else {
                        Aabr::new_empty(Vec2::zero())
                    }
                };
                // Not expecting this case: new_cursor == current_scissor

                // Finish the current command.
                // TODO: ensure we never push empty commands (make fields private & debug assert
                // in constructors?)
                self.draw_commands.push(match self.current_state {
                    State::Plain => DrawCommand::plain(self.start..self.mesh.vertices().len()),
                    State::Image(id) => {
                        DrawCommand::image(self.start..self.mesh.vertices().len(), id)
                    },
                });
                self.start = self.mesh.vertices().len();

                self.draw_commands.push(DrawCommand::Scissor(new_scissor));

                // TODO: support nested clips?
                // TODO: if last command is a clip changing back to the default replace it with
                // this
                // TODO: cull primitives outside the current scissor

                // Renderer child
                self.draw_primitive(*content, offset + clip_offset, renderer);

                // Reset scissor
                self.draw_commands.push(match self.current_state {
                    State::Plain => DrawCommand::plain(self.start..self.mesh.vertices().len()),
                    State::Image(id) => {
                        DrawCommand::image(self.start..self.mesh.vertices().len(), id)
                    },
                });
                self.start = self.mesh.vertices().len();

                self.draw_commands
                    .push(DrawCommand::Scissor(self.window_scissor));
            },
            Primitive::Nothing => {},
        }
    }

    // Switches to the specified state if not already in it
    // If switch occurs current state is converted into a draw command
    fn switch_state(&mut self, state: State) {
        if self.current_state != state {
            let vert_range = self.start..self.mesh.vertices().len();
            let draw_command = match self.current_state {
                State::Plain => DrawCommand::plain(vert_range),
                State::Image(id) => DrawCommand::image(vert_range, id),
            };
            self.draw_commands.push(draw_command);
            self.start = self.mesh.vertices().len();
            self.current_state = state;
        }
    }

    pub fn render(&self, renderer: &mut Renderer, maybe_globals: Option<&Consts<Globals>>) {
        let mut scissor = default_scissor(renderer);
        let globals = maybe_globals.unwrap_or(&self.default_globals);
        let mut locals = &self.interface_locals;
        for draw_command in self.draw_commands.iter() {
            match draw_command {
                DrawCommand::Scissor(new_scissor) => {
                    scissor = *new_scissor;
                },
                DrawCommand::WorldPos(index) => {
                    locals = index.map_or(&self.interface_locals, |i| &self.ingame_locals[i]);
                },
                DrawCommand::Draw { kind, verts } => {
                    let tex = match kind {
                        DrawKind::Image(tex_id) => self.cache.graphic_cache().get_tex(*tex_id),
                        DrawKind::Plain => self.cache.glyph_cache_tex(),
                    };
                    let model = self.model.submodel(verts.clone());
                    renderer.render_ui_element(model, tex, scissor, globals, locals);
                },
            }
        }
    }
}

// Given the the resolution determines the offset needed to align integer
// offsets from the center of the sceen to pixels
#[inline(always)]
fn align(res: Vec2<u16>) -> Vec2<f32> {
    // TODO: does this logic still apply in iced's coordinate system?
    // If the resolution is odd then the center of the screen will be within the
    // middle of a pixel so we need to offset by 0.5 pixels to be on the edge of
    // a pixel
    res.map(|e| (e & 1) as f32 * 0.5)
}

fn default_scissor(renderer: &Renderer) -> Aabr<u16> {
    let (screen_w, screen_h) = renderer.get_resolution().map(|e| e as u16).into_tuple();
    Aabr {
        min: Vec2 { x: 0, y: 0 },
        max: Vec2 {
            x: screen_w,
            y: screen_h,
        },
    }
}

impl iced::Renderer for IcedRenderer {
    // Default styling
    type Defaults = Defaults;
    // TODO: use graph of primitives to enable diffing???
    type Output = (Primitive, iced::mouse::Interaction);

    fn layout<'a, M>(
        &mut self,
        element: &iced::Element<'a, M, Self>,
        limits: &iced::layout::Limits,
    ) -> iced::layout::Node {
        let node = element.layout(self, limits);

        // Trim text measurements cache?

        node
    }
}

// TODO: impl Debugger
