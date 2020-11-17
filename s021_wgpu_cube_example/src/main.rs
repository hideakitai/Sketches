use nannou::math::cgmath;
use nannou::prelude::*;

use std::{iter, mem, num::NonZeroU32, ops::Range, rc::Rc};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub type BufferSize = std::num::NonZeroU64;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        let vertex_size = mem::size_of::<Vertex>();
        let vb_desc = wgpu::VertexBufferDescriptor {
            stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        };
        vb_desc
    }
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>, usize) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec(), index_data.len())
}

fn create_box(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, usize) {
    let (vertex_data, index_data, index_count) = create_vertices();

    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsage::INDEX,
    });

    (vertex_buf, index_buf, index_count)
}

fn create_uniform_buffer(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Buffer {
    let mx_total = Model::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mx_ref),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    });
    uniform_buf
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
        ],
    });
    bind_group_layout
}

fn create_bind_group(
    device: &wgpu::Device,
    uniform_buf: &wgpu::Buffer,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                // resource: uniform_buf.as_entire_binding(),
                resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: None,
    });
    bind_group
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    pipeline_layout
}

fn create_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    vb_desc: &wgpu::VertexBufferDescriptor,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    // let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    // let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));
    let vs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv"));
    let fs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv"));

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
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
            ..Default::default()
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[vb_desc.clone()],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    pipeline
}

fn create_texels(size: usize) -> Vec<u8> {
    use std::iter;

    (0..size * size)
        .flat_map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            iter::once(0xFF - (count * 5) as u8)
                .chain(iter::once(0xFF - (count * 15) as u8))
                .chain(iter::once(0xFF - (count * 50) as u8))
                .chain(iter::once(1))
        })
        .collect()
}

fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size: u32,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
    let texels = create_texels(size as usize);
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth: 1,
    };
    // let texture = device.create_texture(&wgpu::TextureDescriptor {
    //     label: None,
    //     size: texture_extent,
    //     mip_level_count: 1,
    //     sample_count: 1,
    //     dimension: wgpu::TextureDimension::D2,
    //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
    //     usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    // });
    let texture = wgpu::TextureBuilder::new()
        // .label(None)
        .extent(texture_extent)
        .mip_level_count(1)
        .sample_count(1)
        .dimension(wgpu::TextureDimension::D2)
        .format(wgpu::TextureFormat::Rgba8UnormSrgb)
        .usage(wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST)
        .build(device);
    queue.write_texture(
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &texels,
        wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * size,
            rows_per_image: 0,
        },
        texture_extent,
    );
    // let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let texture_view = texture.view().build();
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    (texture, texture_view, sampler)
}

struct Model {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    window_id: WindowId,
    is_capturing: bool,
    capture_begin_frame: u64,
}

impl Model {
    fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
        let mx_view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        mx_correction * mx_projection * mx_view
    }
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

fn model(app: &App) -> Model {
    // build window
    let window_id = app
        .new_window()
        .size(800, 600)
        .title("nannou")
        .key_pressed(key_pressed)
        .resized(resized)
        .raw_view(raw_view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(window_id).unwrap(); // window reference
    let device = window.swap_chain_device(); // reference of gpu device
    let queue = window.swap_chain_queue();
    let sc_desc = window.swap_chain_descriptor();

    use std::mem;

    let (vertex_buf, index_buf, index_count) = create_box(device);
    let uniform_buf = create_uniform_buffer(device, sc_desc);

    let (texture, texture_view, sampler) = create_texture(device, queue, 256);

    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(
        device,
        &uniform_buf,
        &texture_view,
        &sampler,
        &bind_group_layout,
    );

    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);

    // create the render pipeline
    let vb_desc = Vertex::desc();
    // let vertex_state = wgpu::VertexStateDescriptor {
    //     index_format: wgpu::IndexFormat::Uint16,
    //     vertex_buffers: &[wgpu::VertexBufferDescriptor {
    //         stride: vertex_size as wgpu::BufferAddress,
    //         step_mode: wgpu::InputStepMode::Vertex,
    //         attributes: &[
    //             wgpu::VertexAttributeDescriptor {
    //                 format: wgpu::VertexFormat::Float4,
    //                 offset: 0,
    //                 shader_location: 0,
    //             },
    //             wgpu::VertexAttributeDescriptor {
    //                 format: wgpu::VertexFormat::Float2,
    //                 offset: 4 * 4,
    //                 shader_location: 1,
    //             },
    //         ],
    //     }],
    // };

    let pipeline = create_pipeline(device, sc_desc, &vb_desc, &pipeline_layout);

    // let pipeline_wire = if device
    //     .features()
    //     .contains(wgt::Features::NON_FILL_POLYGON_MODE)
    // {
    //     // let fs_wire_module = device.create_shader_module(wgpu::include_spirv!("wire.frag.spv"));
    //     let fs_wire_module =
    //         wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/wire.frag.spv"));
    //     let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    //         label: None,
    //         layout: Some(&pipeline_layout),
    //         vertex_stage: wgpu::ProgrammableStageDescriptor {
    //             module: &vs_module,
    //             entry_point: "main",
    //         },
    //         fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
    //             module: &fs_wire_module,
    //             entry_point: "main",
    //         }),
    //         rasterization_state: Some(wgpu::RasterizationStateDescriptor {
    //             front_face: wgpu::FrontFace::Ccw,
    //             cull_mode: wgpu::CullMode::Back,
    //             // polygon_mode: wgpu::PolygonMode::Line,
    //             ..Default::default()
    //         }),
    //         primitive_topology: wgpu::PrimitiveTopology::TriangleList,
    //         color_states: &[wgpu::ColorStateDescriptor {
    //             format: sc_desc.format,
    //             color_blend: wgpu::BlendDescriptor {
    //                 operation: wgpu::BlendOperation::Add,
    //                 src_factor: wgpu::BlendFactor::SrcAlpha,
    //                 dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    //             },
    //             alpha_blend: wgpu::BlendDescriptor::REPLACE,
    //             write_mask: wgpu::ColorWrite::ALL,
    //         }],
    //         depth_stencil_state: None,
    //         vertex_state,
    //         sample_count: 1,
    //         sample_mask: !0,
    //         alpha_to_coverage_enabled: false,
    //     });
    //     Some(pipeline_wire)
    // } else {
    //     None
    // };
    let pipeline_wire = None;

    let is_capturing = false;
    let capture_begin_frame = 0;

    // Done
    Model {
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        uniform_buf,
        pipeline,
        pipeline_wire,
        window_id,
        is_capturing,
        capture_begin_frame,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn resized(app: &App, model: &mut Model, _: Vector2) {
    let window = app.window(model.window_id).unwrap();
    let sc_desc = window.swap_chain_descriptor();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();

    let mx_total = Model::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    queue.write_buffer(&model.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if let Key::Space = key {
        model.is_capturing = !model.is_capturing;
        if model.is_capturing {
            if model.capture_begin_frame == 0 {
                model.capture_begin_frame = app.elapsed_frames();
            }
            println!("capture start");
        } else {
            println!("capture finished");
        }
    }
}

fn raw_view(app: &App, model: &Model, raw_frame: RawFrame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &raw_frame.swap_chain_texture(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&model.pipeline);
        rpass.set_bind_group(0, &model.bind_group, &[]);
        rpass.set_index_buffer(model.index_buf.slice(..));
        rpass.set_vertex_buffer(0, model.vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..model.index_count as u32, 0, 0..1);
        if let Some(ref pipe) = model.pipeline_wire {
            rpass.set_pipeline(pipe);
            rpass.draw_indexed(0..model.index_count as u32, 0, 0..1);
        }
    }

    queue.submit(Some(encoder.finish()));

    // TODO:
    if model.is_capturing {
        let num_capture_frame = app.elapsed_frames() - model.capture_begin_frame;
        let file_path = app
            .project_path()
            .unwrap()
            .join(format!("{:05}", num_capture_frame))
            .with_extension("png");
        app.main_window().capture_frame(file_path);
    }
}

// // conversion to movie file looks like:
// // ffmpeg -framerate 30 -i "%5d.png" -pix_fmt yuv420p output.mp4
// fn captured_frame_path(app: &App, num_frame: u64) -> std::path::PathBuf {}
