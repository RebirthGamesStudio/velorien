use super::graphic::{Graphic, GraphicCache, Id as GraphicId};
use crate::{
    render::{Mesh, Renderer, Texture, UiPipeline},
    Error,
};
use conrod_core::{text::GlyphCache, widget::Id};
use hashbrown::HashMap;
use vek::*;

// Multiplied by current window size
const GLYPH_CACHE_SIZE: u16 = 1;
// Glyph cache tolerances
const SCALE_TOLERANCE: f32 = 0.5;
const POSITION_TOLERANCE: f32 = 0.5;

type TextCache = HashMap<Id, Mesh<UiPipeline>>;

pub struct Cache {
    // Map from text ids to their positioned glyphs.
    text_cache: TextCache,
    glyph_cache: GlyphCache<'static>,
    glyph_cache_tex: Texture,
    graphic_cache: GraphicCache,
}

// TODO: Should functions be returning UiError instead of Error?
impl Cache {
    pub fn new(renderer: &mut Renderer) -> Result<Self, Error> {
        let (w, h) = renderer.get_resolution().into_tuple();

        let max_texture_size = renderer.max_texture_size();

        let glyph_cache_dims =
            Vec2::new(w, h).map(|e| (e * GLYPH_CACHE_SIZE).min(max_texture_size).max(512));

        Ok(Self {
            text_cache: Default::default(),
            glyph_cache: GlyphCache::builder()
                .dimensions(glyph_cache_dims.x as u32, glyph_cache_dims.y as u32)
                .scale_tolerance(SCALE_TOLERANCE)
                .position_tolerance(POSITION_TOLERANCE)
                .build(),
            glyph_cache_tex: renderer.create_dynamic_texture(glyph_cache_dims.map(|e| e as u16))?,
            graphic_cache: GraphicCache::new(renderer),
        })
    }

    pub fn glyph_cache_tex(&self) -> &Texture { &self.glyph_cache_tex }

    pub fn cache_mut_and_tex(
        &mut self,
    ) -> (
        &mut GraphicCache,
        &mut TextCache,
        &mut GlyphCache<'static>,
        &Texture,
    ) {
        (
            &mut self.graphic_cache,
            &mut self.text_cache,
            &mut self.glyph_cache,
            &self.glyph_cache_tex,
        )
    }

    pub fn graphic_cache(&self) -> &GraphicCache { &self.graphic_cache }

    pub fn add_graphic(&mut self, graphic: Graphic) -> GraphicId {
        self.graphic_cache.add_graphic(graphic)
    }

    pub fn replace_graphic(&mut self, id: GraphicId, graphic: Graphic) {
        self.graphic_cache.replace_graphic(id, graphic)
    }

    /// Resizes and clears the various caches.
    ///
    /// To be called when something like the scaling factor changes,
    /// invalidating all existing cached UI state.
    pub fn resize(&mut self, renderer: &mut Renderer) -> Result<(), Error> {
        self.graphic_cache.clear_cache(renderer);
        self.text_cache.clear();
        let max_texture_size = renderer.max_texture_size();
        let cache_dims = renderer
            .get_resolution()
            .map(|e| (e * GLYPH_CACHE_SIZE).min(max_texture_size).max(512));
        self.glyph_cache = GlyphCache::builder()
            .dimensions(cache_dims.x as u32, cache_dims.y as u32)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build();
        self.glyph_cache_tex = renderer.create_dynamic_texture(cache_dims.map(|e| e as u16))?;
        Ok(())
    }
}
