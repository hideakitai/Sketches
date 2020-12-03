use nannou::prelude::*;

pub mod buffer;
pub mod camera;
pub mod geom;
pub mod instance;
pub mod light;
pub mod texture;

pub trait Pass {
    fn create_render_pipeline_layout(
        device: &wgpu::Device,
        layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: layouts,
            push_constant_ranges: &[],
        })
    }

    // generic render pipiline generator
    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_descs: &[wgpu::VertexBufferDescriptor],
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        // wgpu::RenderPipelineBuilder::from_layout(layout, vs_module)
        //     // .label(Some("Render Pipeline"))
        //     .fragment_shader(fs_module)
        //     .front_face(wgpu::FrontFace::Ccw)
        //     .cull_mode(wgpu::CullMode::Back)
        //     .depth_bias(0)
        //     .depth_bias_slope_scale(0.)
        //     .depth_bias_clamp(0.)
        //     .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
        //     .color_format(color_format)
        //     .color_blend(wgpu::BlendDescriptor::REPLACE)
        //     .alpha_blend(wgpu::BlendDescriptor::REPLACE)
        //     .write_mask(wgpu::ColorWrite::ALL)
        //     .depth_format(depth_format.unwrap())
        //     .depth_write_enabled(true)
        //     .depth_compare(wgpu::CompareFunction::Less)
        //     .sample_count(1)
        //     .index_format(wgpu::IndexFormat::Uint32)
        //     .add_vertex_buffer_descriptor(vertex_descs) // lifetime error...
        //     .build(device)

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"), // TODO:
            layout: Some(&layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: color_format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: depth_format.map(|format| wgpu::DepthStencilStateDescriptor {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: vertex_descs,
            },
        })
    }
}
