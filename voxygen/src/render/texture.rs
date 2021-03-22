use super::{gfx_backend, RenderError};
use gfx::{self, traits::Factory};
use image::{DynamicImage, GenericImageView};
use vek::Vec2;

type DefaultShaderFormat = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);

/// Represents an image that has been uploaded to the GPU.
#[derive(Clone)]
pub struct Texture<F: gfx::format::Formatted = DefaultShaderFormat>
where
    F::Surface: gfx::format::TextureSurface,
    F::Channel: gfx::format::TextureChannel,
    <F::Surface as gfx::format::SurfaceTyped>::DataType: Copy,
{
    pub tex: gfx::handle::Texture<gfx_backend::Resources, <F as gfx::format::Formatted>::Surface>,
    pub srv: gfx::handle::ShaderResourceView<
        gfx_backend::Resources,
        <F as gfx::format::Formatted>::View,
    >,
    pub sampler: gfx::handle::Sampler<gfx_backend::Resources>,
}

impl<F: gfx::format::Formatted> Texture<F>
where
    F::Surface: gfx::format::TextureSurface,
    F::Channel: gfx::format::TextureChannel,
    <F::Surface as gfx::format::SurfaceTyped>::DataType: Copy,
{
    pub fn new(
        factory: &mut gfx_backend::Factory,
        image: &DynamicImage,
        filter_method: Option<gfx::texture::FilterMethod>,
        wrap_mode: Option<gfx::texture::WrapMode>,
        border: Option<gfx::texture::PackedColor>,
    ) -> Result<Self, RenderError> {
        // TODO: Actualy handle images that aren't in rgba format properly.
        let buffer = image.as_flat_samples_u8().ok_or_else(|| {
            RenderError::CustomError(
                "We currently do not support color formats using more than 4 bytes / pixel.".into(),
            )
        })?;
        let (tex, srv) = factory
            .create_texture_immutable_u8::<F>(
                gfx::texture::Kind::D2(
                    image.width() as u16,
                    image.height() as u16,
                    gfx::texture::AaMode::Single,
                ),
                gfx::texture::Mipmap::Provided,
                // Guarenteed to be correct, since all the conversions from DynamicImage to
                // FlatSamples<u8> go through the underlying ImageBuffer's implementation of
                // as_flat_samples(), which guarantees that the resulting FlatSamples is
                // well-formed.
                &[buffer.as_slice()],
            )
            .map_err(RenderError::CombinedError)?;

        let mut sampler_info = gfx::texture::SamplerInfo::new(
            filter_method.unwrap_or(gfx::texture::FilterMethod::Scale),
            wrap_mode.unwrap_or(gfx::texture::WrapMode::Clamp),
        );
        let transparent = [0.0, 0.0, 0.0, 1.0].into();
        sampler_info.border = border.unwrap_or(transparent);
        Ok(Self {
            tex,
            srv,
            sampler: factory.create_sampler(sampler_info),
        })
    }

    pub fn new_dynamic(
        factory: &mut gfx_backend::Factory,
        width: u16,
        height: u16,
    ) -> Result<Self, RenderError> {
        let tex = factory.create_texture(
            gfx::texture::Kind::D2(
                width,
                height,
                gfx::texture::AaMode::Single,
            ),
            1_u8,
            gfx::memory::Bind::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic,
            Some(<<F as gfx::format::Formatted>::Channel as gfx::format::ChannelTyped>::get_channel_type()),
        )
            .map_err(|err| RenderError::CombinedError(gfx::CombinedError::Texture(err)))?;

        let srv = factory
            .view_texture_as_shader_resource::<F>(&tex, (0, 0), gfx::format::Swizzle::new())
            .map_err(|err| RenderError::CombinedError(gfx::CombinedError::Resource(err)))?;

        Ok(Self {
            tex,
            srv,
            sampler: factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
                gfx::texture::WrapMode::Clamp,
            )),
        })
    }

    pub fn new_immutable_raw(
        factory: &mut gfx_backend::Factory,
        kind: gfx::texture::Kind,
        mipmap: gfx::texture::Mipmap,
        data: &[&[<F::Surface as gfx::format::SurfaceTyped>::DataType]],
        sampler_info: gfx::texture::SamplerInfo,
    ) -> Result<Self, RenderError> {
        let (tex, srv) = factory
            .create_texture_immutable::<F>(kind, mipmap, data)
            .map_err(RenderError::CombinedError)?;

        Ok(Self {
            tex,
            srv,
            sampler: factory.create_sampler(sampler_info),
        })
    }

    pub fn new_raw(
        _device: &mut gfx_backend::Device,
        factory: &mut gfx_backend::Factory,
        kind: gfx::texture::Kind,
        max_levels: u8,
        bind: gfx::memory::Bind,
        usage: gfx::memory::Usage,
        levels: (u8, u8),
        swizzle: gfx::format::Swizzle,
        sampler_info: gfx::texture::SamplerInfo,
    ) -> Result<Self, RenderError> {
        let tex = factory
            .create_texture(
                kind,
                max_levels as gfx::texture::Level,
                bind | gfx::memory::Bind::SHADER_RESOURCE,
                usage,
                Some(<<F as gfx::format::Formatted>::Channel as gfx::format::ChannelTyped>::get_channel_type())
            )
            .map_err(|err| RenderError::CombinedError(gfx::CombinedError::Texture(err)))?;

        let srv = factory
            .view_texture_as_shader_resource::<F>(&tex, levels, swizzle)
            .map_err(|err| RenderError::CombinedError(gfx::CombinedError::Resource(err)))?;

        Ok(Self {
            tex,
            srv,
            sampler: factory.create_sampler(sampler_info),
        })
    }

    /// Update a texture with the given data (used for updating the glyph cache
    /// texture).

    pub fn update(
        &self,
        encoder: &mut gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[<F::Surface as gfx::format::SurfaceTyped>::DataType],
    ) -> Result<(), RenderError> {
        let info = gfx::texture::ImageInfoCommon {
            xoffset: offset[0],
            yoffset: offset[1],
            zoffset: 0,
            width: size[0],
            height: size[1],
            depth: 0,
            format: (),
            mipmap: 0,
        };
        encoder
            .update_texture::<<F as gfx::format::Formatted>::Surface, F>(
                &self.tex, None, info, data,
            )
            .map_err(RenderError::TexUpdateError)
    }

    /// Get dimensions of the represented image.
    pub fn get_dimensions(&self) -> Vec2<u16> {
        let (w, h, ..) = self.tex.get_info().kind.get_dimensions();
        Vec2::new(w, h)
    }
}
