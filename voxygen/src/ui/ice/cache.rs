use super::{
    graphic::{Graphic, GraphicCache, Id as GraphicId},
    renderer::{IcedRenderer, Primitive},
};
use crate::{
    render::{Renderer, Texture},
    Error,
};
use glyph_brush::{
    GlyphBrushBuilder, GlyphCalculator, GlyphCalculatorBuilder, GlyphCalculatorGuard,
};
use std::cell::RefCell;
use vek::*;

// Multiplied by current window size
const GLYPH_CACHE_SIZE: u16 = 1;
// Glyph cache tolerances
const SCALE_TOLERANCE: f32 = 0.1;
const POSITION_TOLERANCE: f32 = 0.1;

type GlyphBrush = glyph_brush::GlyphBrush<'static, (Aabr<f32>, Aabr<f32>)>;

pub type Font = glyph_brush::rusttype::Font<'static>;

pub struct Cache {
    glyph_cache: GlyphBrush,
    glyph_cache_tex: Texture,
    graphic_cache: GraphicCache,
}

// TODO: Should functions be returning UiError instead of Error?
impl Cache {
    pub fn new(renderer: &mut Renderer, default_font: Font) -> Result<Self, Error> {
        let (w, h) = renderer.get_resolution().into_tuple();

        let max_texture_size = renderer.max_texture_size();

        let glyph_cache_dims =
            Vec2::new(w, h).map(|e| (e * GLYPH_CACHE_SIZE).min(max_texture_size as u16).max(512));

        Ok(Self {
            glyph_cache: GlyphBrushBuilder::using_font(default_font)
                .initial_cache_size((glyph_cache_dims.x as u32, glyph_cache_dims.y as u32))
                .gpu_cache_scale_tolerance(SCALE_TOLERANCE)
                .gpu_cache_position_tolerance(POSITION_TOLERANCE)
                .build(),
            glyph_cache_tex: renderer.create_dynamic_texture(glyph_cache_dims.map(|e| e as u16))?,
            graphic_cache: GraphicCache::new(renderer),
        })
    }

    pub fn glyph_cache_tex(&self) -> &Texture { &self.glyph_cache_tex }

    pub fn glyph_cache_mut_and_tex(&mut self) -> (&mut GlyphBrush, &Texture) {
        (&mut self.glyph_cache, &self.glyph_cache_tex)
    }

    pub fn glyph_cache_mut(&mut self) -> &mut GlyphBrush { &mut self.glyph_cache }

    // TODO: add font fn

    pub fn graphic_cache(&self) -> &GraphicCache { &self.graphic_cache }

    pub fn graphic_cache_mut(&mut self) -> &mut GraphicCache { &mut self.graphic_cache }

    pub fn add_graphic(&mut self, graphic: Graphic) -> GraphicId {
        self.graphic_cache.add_graphic(graphic)
    }

    pub fn replace_graphic(&mut self, id: GraphicId, graphic: Graphic) {
        self.graphic_cache.replace_graphic(id, graphic)
    }

    // Resizes and clears the GraphicCache
    pub fn resize_graphic_cache(&mut self, renderer: &mut Renderer) {
        self.graphic_cache.clear_cache(renderer);
    }

    // Resizes and clears the GlyphCache
    pub fn resize_glyph_cache(&mut self, renderer: &mut Renderer) -> Result<(), Error> {
        let max_texture_size = renderer.max_texture_size();
        let cache_dims = renderer
            .get_resolution()
            .map(|e| (e * GLYPH_CACHE_SIZE).min(max_texture_size as u16).max(512));
        self.glyph_cache = self
            .glyph_cache
            .to_builder()
            .initial_cache_size((cache_dims.x as u32, cache_dims.y as u32))
            .build();
        self.glyph_cache_tex = renderer.create_dynamic_texture(cache_dims.map(|e| e as u16))?;
        Ok(())
    }
}

pub struct GlyphCalcCache {
    // Hold one of these for adding new fonts
    builder: GlyphCalculatorBuilder<'static>,
    calculator: GlyphCalculator<'static>,
}

impl GlyphCalcCache {
    pub fn new(default_font: Font) -> Self {
        // Multiple copies of the font in memory :/ (using Arc<[u8]> might help)
        let builder = GlyphCalculatorBuilder::using_font(default_font);
        let calculator = builder.clone().build();

        Self {
            builder,
            calculator,
        }
    }

    pub fn frame_guard<'a>(&'a self) -> GlyphCalculatorGuard<'a, 'static> {
        self.calculator.cache_scope()
    }

    // add new font fn
}

pub struct FrameRenderer<'a> {
    pub renderer: &'a mut IcedRenderer,
    pub glyph_calc: RefCell<GlyphCalculatorGuard<'a, 'static>>,
}

impl<'a> FrameRenderer<'a> {
    pub fn new(renderer: &'a mut IcedRenderer, glyph_calc_cache: &'a mut GlyphCalcCache) -> Self {
        Self {
            renderer,
            glyph_calc: RefCell::new(glyph_calc_cache.frame_guard()),
        }
    }
}

impl iced::Renderer for FrameRenderer<'_> {
    // Default styling
    type Defaults = ();
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
