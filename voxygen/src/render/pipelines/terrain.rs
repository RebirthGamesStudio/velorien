use super::super::{AaMode, GlobalsLayouts};
use vek::*;
use zerocopy::AsBytes;

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct Vertex {
    pos_norm: u32,
    atlas_pos: u32,
}

impl Vertex {
    #[allow(clippy::identity_op)] // TODO: Pending review in #587
    /// NOTE: meta is true when the terrain vertex is touching water.
    pub fn new(atlas_pos: Vec2<u16>, pos: Vec3<f32>, norm: Vec3<f32>, meta: bool) -> Self {
        const EXTRA_NEG_Z: f32 = 32768.0;

        let norm_bits = if norm.x != 0.0 {
            if norm.x < 0.0 { 0 } else { 1 }
        } else if norm.y != 0.0 {
            if norm.y < 0.0 { 2 } else { 3 }
        } else if norm.z < 0.0 {
            4
        } else {
            5
        };
        Self {
            pos_norm: ((pos.x as u32) & 0x003F) << 0
                | ((pos.y as u32) & 0x003F) << 6
                | (((pos + EXTRA_NEG_Z).z.max(0.0).min((1 << 16) as f32) as u32) & 0xFFFF) << 12
                | if meta { 1 } else { 0 } << 28
                | (norm_bits & 0x7) << 29,
            atlas_pos: ((atlas_pos.x as u32) & 0xFFFF) << 0 | ((atlas_pos.y as u32) & 0xFFFF) << 16,
        }
    }

    pub fn new_figure(atlas_pos: Vec2<u16>, pos: Vec3<f32>, norm: Vec3<f32>, bone_idx: u8) -> Self {
        let norm_bits = if norm.x.min(norm.y).min(norm.z) < 0.0 {
            0
        } else {
            1
        };
        let axis_bits = if norm.x != 0.0 {
            0
        } else if norm.y != 0.0 {
            1
        } else {
            2
        };
        Self {
            pos_norm: pos
                .map2(Vec3::new(0, 9, 18), |e, shift| {
                    (((e * 2.0 + 256.0) as u32) & 0x1FF) << shift
                })
                .reduce_bitor()
                | (((bone_idx & 0xF) as u32) << 27)
                | (norm_bits << 31),
            atlas_pos: ((atlas_pos.x as u32) & 0x7FFF) << 2
                | ((atlas_pos.y as u32) & 0x7FFF) << 17
                | axis_bits & 3,
        }
    }

    pub fn make_col_light(
        // 0 to 31
        light: u8,
        // 0 to 31
        glow: u8,
        col: Rgb<u8>,
    ) -> [u8; 4] {
        //[col.r, col.g, col.b, light]
        // It would be nice for this to be cleaner, but we want to squeeze 5 fields into
        // 4. We can do this because both `light` and `glow` go from 0 to 31,
        // meaning that they can both fit into 5 bits. If we steal a bit from
        // red and blue each (not green, human eyes are more sensitive to
        // changes in green) then we get just enough to expand the nibbles of
        // the alpha field enough to fit both `light` and `glow`.
        //
        // However, we now have a problem. In the shader code with use hardware
        // filtering to get at the `light` and `glow` attributes (but not
        // colour, that remains constant across a block). How do we resolve this
        // if we're twiddling bits? The answer is to very carefully manipulate
        // the bit pattern such that the fields we want to filter (`light` and
        // `glow`) always sit as the higher bits of the fields. Then, we can do
        // some modulation magic to extract them from the filtering unharmed and use
        // unfiltered texture access (i.e: `texelFetch`) to access the colours, plus a
        // little bit-fiddling.
        //
        // TODO: This isn't currently working (no idea why). See `srgb.glsl` for current
        // impl that intead does manual bit-twiddling and filtering.
        [
            (light.min(31) << 3) | ((col.r >> 1) & 0b111),
            (glow.min(31) << 3) | ((col.b >> 1) & 0b111),
            (col.r & 0b11110000) | (col.b >> 4),
            col.g, // Green is lucky, it remains unscathed
        ]
    }

    /// Set the bone_idx for an existing figure vertex.
    pub fn set_bone_idx(&mut self, bone_idx: u8) {
        self.pos_norm = (self.pos_norm & !(0xF << 27)) | ((bone_idx as u32 & 0xF) << 27);
    }

    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Uint,1 => Uint],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct Locals {
    model_offs: [f32; 3],
    load_time: f32,
    atlas_offs: [i32; 4],
}

impl Locals {
    pub fn default() -> Self {
        Self {
            model_offs: [0.0; 3],
            load_time: 0.0,
            atlas_offs: [0; 4],
        }
    }
}

pub struct TerrainLayout {
    pub locals: wgpu::BindGroupLayout,
}

impl TerrainLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            locals: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // locals
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // col lights
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
            }),
        }
    }
}

pub struct TerrainPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl TerrainPipeline {
    pub fn new(
        device: &wgpu::Device,
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        sc_desc: &wgpu::SwapChainDescriptor,
        global_layout: &GlobalsLayouts,
        layout: &TerrainLayout,
        aa_mode: AaMode,
    ) -> Self {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Terrain pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&global_layout.globals, &layout.locals],
            });

        let samples = match aa_mode {
            AaMode::None | AaMode::Fxaa => 1,
            // TODO: Ensure sampling in the shader is exactly between the 4 texels
            AaMode::SsaaX4 => 1,
            AaMode::MsaaX4 => 4,
            AaMode::MsaaX8 => 8,
            AaMode::MsaaX16 => 16,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: samples,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            pipeline: render_pipeline,
        }
    }
}
