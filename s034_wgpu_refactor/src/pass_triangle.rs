use nannou::prelude::*;
use std::time::Duration;

use crate::renderer;

pub struct PassTriangle {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    render_pipeline: wgpu::RenderPipeline,
}

impl PassTriangle {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let texture = wgpu::TextureBuilder::new()
            .extent(wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            })
            // .label(None)
            // .array_layer_count(1)
            .mip_level_count(1)
            .sample_count(1)
            .dimension(wgpu::TextureDimension::D2)
            .format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .usage(wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT)
            .build(device);

        let texture_view = texture.view().build();

        let vs_module =
            &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/offline.vert.spv"));
        let fs_module =
            &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/offline.frag.spv"));

        let render_pipeline_layout = renderer::PipelineLayoutBuilder::new()
            .label("Render Pipeline Layout")
            .build(device);

        let render_pipeline =
            wgpu::RenderPipelineBuilder::from_layout(&render_pipeline_layout, vs_module)
                .fragment_shader(fs_module)
                // .front_face(wgpu::FrontFace::Ccw)
                .cull_mode(wgpu::CullMode::Back)
                // .depth_bias(0)
                // .depth_bias_slope_scale(0.0)
                // .depth_bias_clamp(0.0)
                // .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
                .color_format(texture.format())
                .color_blend(wgpu::BlendDescriptor::REPLACE)
                .alpha_blend(wgpu::BlendDescriptor::REPLACE)
                // .write_mask(wgpu::ColorWrite::ALL)
                // .depth_stencil_state(None)
                .index_format(wgpu::IndexFormat::Uint16)
                // .add_vertex_buffer(&[])
                .sample_count(1)
                // .sample_mask(!0)
                .build(device);

        Self {
            texture,
            texture_view,
            render_pipeline,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {}

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, i: &i32) {
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(&self.texture_view, |color| {
                color
                    .resolve_target(None)
                    .load_op(wgpu::LoadOp::Clear(wgpu::Color {
                        r: *i as f64 / 10.,
                        g: *i as f64 / 20.,
                        b: 0.5,
                        a: 1.0,
                    }))
                    .store_op(true)
            })
            .begin(encoder);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    pub fn resized(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {}

    pub fn key_pressed(&mut self, key: Key) {}

    pub fn key_released(&mut self, key: Key) {}

    pub fn mouse_moved(&mut self, curr_pos: Point2, prev_pos: Point2) {}

    pub fn mouse_wheel(&mut self, delta: &MouseScrollDelta) {}

    pub fn output_texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
