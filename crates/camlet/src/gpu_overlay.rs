//! WGPU compositor used by the live overlay.

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use camlet_camera::VideoFrame;
use camlet_core::appearance::{AppearanceSettings, HexColor, OverlayShape};
use camlet_core::geometry::{Rect, fit_frame, shape_bounds_for_source};
use iced::mouse;
use iced::widget::shader::{self, Viewport};
use iced::{Rectangle, wgpu};
use num_traits::ToPrimitive;

const EDGE_MARGIN_PHYSICAL_PIXELS: f64 = 2.0;

/// A borrowed live-overlay scene. Drawing creates an owned primitive for Iced.
#[derive(Debug, Clone, Copy)]
pub struct OverlayProgram<'a> {
    source: &'a VideoFrame,
    appearance: &'a AppearanceSettings,
    frame_revision: u64,
}

impl<'a> OverlayProgram<'a> {
    /// Creates a live GPU scene.
    #[must_use]
    pub const fn new(
        source: &'a VideoFrame,
        appearance: &'a AppearanceSettings,
        frame_revision: u64,
    ) -> Self {
        Self {
            source,
            appearance,
            frame_revision,
        }
    }
}

/// Rejects malformed frame dimensions before any pixels reach WGPU.
///
/// # Errors
///
/// Returns a stable message for empty, overflowing, or truncated RGBA frames.
pub fn validate_source_frame(source: &VideoFrame) -> Result<(), &'static str> {
    let expected = usize::try_from(source.width)
        .ok()
        .and_then(|width| {
            usize::try_from(source.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4));
    if source.width == 0 || source.height == 0 || expected != Some(source.rgba.len()) {
        return Err("invalid source frame");
    }
    Ok(())
}

impl<Message> shader::Program<Message> for OverlayProgram<'_> {
    type State = ();
    type Primitive = OverlayPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        OverlayPrimitive {
            source_width: self.source.width,
            source_height: self.source.height,
            frame_revision: self.frame_revision,
            rgba: self.source.rgba.clone(),
            appearance: self.appearance.clone(),
        }
    }
}

/// One owned frame submitted to the shared WGPU pipeline.
#[derive(Debug)]
pub struct OverlayPrimitive {
    source_width: u32,
    source_height: u32,
    frame_revision: u64,
    rgba: Vec<u8>,
    appearance: AppearanceSettings,
}

impl shader::Primitive for OverlayPrimitive {
    type Pipeline = OverlayPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        pipeline.prepare(device, queue, bounds, viewport, self);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some(bind_group) = pipeline.bind_group.as_ref() else {
            return true;
        };
        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        true
    }
}

/// Shared GPU resources for the live compositor.
pub struct OverlayPipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    texture_size: (u32, u32),
    uploaded_revision: Option<u64>,
}

impl shader::Pipeline for OverlayPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self::new(device, format)
    }
}

impl OverlayPipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camlet overlay bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("camlet overlay pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("camlet overlay shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("overlay.wgsl"))),
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("camlet overlay render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("camlet camera sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camlet overlay uniforms"),
            size: std::mem::size_of::<RawUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            texture: None,
            bind_group: None,
            texture_size: (0, 0),
            uploaded_revision: None,
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
        primitive: &OverlayPrimitive,
    ) {
        if self.texture_size != (primitive.source_width, primitive.source_height) {
            self.create_texture(device, primitive.source_width, primitive.source_height);
        }
        if self.uploaded_revision != Some(primitive.frame_revision)
            && let Some(texture) = self.texture.as_ref()
        {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &primitive.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(primitive.source_width * 4),
                    rows_per_image: Some(primitive.source_height),
                },
                wgpu::Extent3d {
                    width: primitive.source_width,
                    height: primitive.source_height,
                    depth_or_array_layers: 1,
                },
            );
            self.uploaded_revision = Some(primitive.frame_revision);
        }

        let physical_width = bounds.width * viewport.scale_factor();
        let physical_height = bounds.height * viewport.scale_factor();
        let uniforms = RawUniforms::new(
            &primitive.appearance,
            primitive.source_width,
            primitive.source_height,
            physical_width,
            physical_height,
        );
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn create_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.texture = None;
            self.bind_group = None;
            self.texture_size = (0, 0);
            return;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("camlet camera frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camlet overlay bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        }));
        self.texture = Some(texture);
        self.texture_size = (width, height);
        self.uploaded_revision = None;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct RawUniforms {
    surface_size: [f32; 2],
    frame_offset: [f32; 2],
    frame_size: [f32; 2],
    ring_corner: [f32; 2],
    shape_bounds: [f32; 4],
    shape: u32,
    _padding_0: u32,
    _padding_1: u32,
    _padding_2: u32,
    ring_color: [f32; 4],
    accent_color: [f32; 4],
}

impl RawUniforms {
    fn new(
        appearance: &AppearanceSettings,
        source_width: u32,
        source_height: u32,
        physical_width: f32,
        physical_height: f32,
    ) -> Self {
        let surface_size = physical_width.min(physical_height).max(1.0);
        let scale = surface_size / f32::from(appearance.size);
        let visible = inset_rect(
            scale_rect(
                shape_bounds_for_source(
                    appearance.shape,
                    appearance.size,
                    source_width,
                    source_height,
                ),
                f64::from(scale),
            ),
            EDGE_MARGIN_PHYSICAL_PIXELS,
        );
        let ring_width = f32::from(appearance.ring_thickness) * scale;
        let frame_bounds = inset_rect(visible, f64::from(ring_width.max(0.0)));
        let transform = fit_frame(source_width, source_height, frame_bounds, appearance.fit)
            .unwrap_or(camlet_core::geometry::FrameTransform {
                scale: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
                width: 1.0,
                height: 1.0,
            });
        Self {
            surface_size: [physical_width.max(1.0), physical_height.max(1.0)],
            frame_offset: [
                transform.offset_x.to_f32().unwrap_or(0.0),
                transform.offset_y.to_f32().unwrap_or(0.0),
            ],
            frame_size: [
                transform.width.to_f32().unwrap_or(1.0),
                transform.height.to_f32().unwrap_or(1.0),
            ],
            ring_corner: [ring_width, f32::from(appearance.corner_roundness) * scale],
            shape_bounds: [
                visible.x.to_f32().unwrap_or(0.0),
                visible.y.to_f32().unwrap_or(0.0),
                visible.width.to_f32().unwrap_or(surface_size),
                visible.height.to_f32().unwrap_or(surface_size),
            ],
            shape: shape_id(appearance.shape),
            _padding_0: 0,
            _padding_1: 0,
            _padding_2: 0,
            ring_color: color(appearance.ring_color),
            accent_color: color(appearance.ring_accent_color),
        }
    }
}

const fn shape_id(shape: OverlayShape) -> u32 {
    match shape {
        OverlayShape::Original => 0,
        OverlayShape::Circle => 1,
        OverlayShape::RoundedSquare => 2,
        OverlayShape::Diamond => 3,
        OverlayShape::RectangleY => 4,
        OverlayShape::RectangleX => 5,
    }
}

fn color(value: HexColor) -> [f32; 4] {
    [
        f32::from(value.red) / 255.0,
        f32::from(value.green) / 255.0,
        f32::from(value.blue) / 255.0,
        f32::from(value.alpha) / 255.0,
    ]
}

fn scale_rect(rectangle: Rect, scale: f64) -> Rect {
    Rect {
        x: rectangle.x * scale,
        y: rectangle.y * scale,
        width: rectangle.width * scale,
        height: rectangle.height * scale,
    }
}

fn inset_rect(rectangle: Rect, inset: f64) -> Rect {
    let maximum = (rectangle.width.min(rectangle.height) / 2.0 - 0.5).max(0.0);
    let inset = inset.clamp(0.0, maximum);
    Rect {
        x: rectangle.x + inset,
        y: rectangle.y + inset,
        width: (rectangle.width - inset * 2.0).max(1.0),
        height: (rectangle.height - inset * 2.0).max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use camlet_core::appearance::{AppearanceSettings, OverlayShape, PreviewFitMode};

    use super::RawUniforms;

    #[test]
    fn uniforms_preserve_cover_geometry_and_physical_ring_width() {
        let appearance = AppearanceSettings {
            shape: OverlayShape::Circle,
            size: 224,
            ring_thickness: 4,
            fit: PreviewFitMode::Cover,
            ..AppearanceSettings::default()
        };
        let uniforms = RawUniforms::new(&appearance, 640, 480, 448.0, 448.0);
        for (actual, expected) in [
            (uniforms.ring_corner[0], 8.0),
            (uniforms.shape_bounds[0], 2.0),
            (uniforms.shape_bounds[1], 2.0),
            (uniforms.shape_bounds[2], 444.0),
            (uniforms.shape_bounds[3], 444.0),
            (uniforms.frame_offset[0], -61.333_332),
            (uniforms.frame_offset[1], 10.0),
            (uniforms.frame_size[0], 570.666_7),
            (uniforms.frame_size[1], 428.0),
        ] {
            assert!((actual - expected).abs() < 0.001);
        }
    }

    #[test]
    fn uniform_layout_is_wgsl_aligned() {
        assert_eq!(std::mem::size_of::<RawUniforms>(), 96);
        assert_eq!(std::mem::align_of::<RawUniforms>(), 4);
    }
}
