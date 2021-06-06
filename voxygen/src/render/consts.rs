use super::{gfx_backend, RenderError};
use gfx::{self, traits::FactoryExt};

/// A handle to a series of constants sitting on the GPU. This is used to hold
/// information used in the rendering process that does not change throughout a
/// single render pass.
#[derive(Clone)]
pub struct Consts<T: Copy + gfx::traits::Pod> {
    pub buf: gfx::handle::Buffer<gfx_backend::Resources, T>,
}

impl<T: Copy + gfx::traits::Pod> Consts<T> {
    /// Create a new `Const<T>`.
    pub fn new(factory: &mut gfx_backend::Factory, len: usize) -> Self {
        Self {
            buf: factory.create_constant_buffer(len),
        }
    }

    /// Update the GPU-side value represented by this constant handle.

    pub fn update(
        &mut self,
        encoder: &mut gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
        vals: &[T],
        offset: usize,
    ) -> Result<(), RenderError> {
        if vals.is_empty() {
            Ok(())
        } else {
            encoder
                .update_buffer(&self.buf, vals, offset)
                .map_err(RenderError::UpdateError)
        }
    }
}
