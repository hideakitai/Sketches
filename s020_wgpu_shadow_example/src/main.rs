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
    _pos: [i8; 4],
    _normal: [i8; 4],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        let vertex_size = mem::size_of::<Vertex>();
        let vb_desc = wgpu::VertexBufferDescriptor {
            stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            // attributes: &wgpu::vertex_attr_array![0 => Char4, 1 => Char4],
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Char4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[i8; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Char4,
                },
            ],
        };
        vb_desc
    }
}

struct CubeDesc {
    offset: cgmath::Vector3<f32>,
    angle: f32,
    scale: f32,
    rotation: f32,
}

struct Entity {
    mx_world: cgmath::Matrix4<f32>,
    rotation_speed: f32,
    color: wgpu::Color,
    vertex_buf: Rc<wgpu::Buffer>,
    index_buf: Rc<wgpu::Buffer>,
    index_count: usize,
    uniform_offset: wgpu::DynamicOffset,
}

struct Light {
    pos: cgmath::Point3<f32>,
    color: wgpu::Color,
    fov: f32,
    depth: Range<f32>,
    target_view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightRaw {
    proj: [[f32; 4]; 4],
    pos: [f32; 4],
    color: [f32; 4],
}

impl Light {
    fn to_raw(&self) -> LightRaw {
        use cgmath::{Deg, EuclideanSpace, Matrix4, PerspectiveFov, Point3, Vector3};

        let mx_view = Matrix4::look_at(self.pos, Point3::origin(), Vector3::unit_z());
        let projection = PerspectiveFov {
            fovy: Deg(self.fov).into(),
            aspect: 1.0,
            near: self.depth.start,
            far: self.depth.end,
        };
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        let mx_view_proj =
            mx_correction * cgmath::Matrix4::from(projection.to_perspective()) * mx_view;
        LightRaw {
            proj: *mx_view_proj.as_ref(),
            pos: [self.pos.x, self.pos.y, self.pos.z, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ForwardUniforms {
    proj: [[f32; 4]; 4],
    num_lights: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct EntityUniforms {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

#[repr(C)]
struct ShadowUniforms {
    proj: [[f32; 4]; 4],
}

struct Pass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

struct Model {
    entities: Vec<Entity>,
    lights: Vec<Light>,
    lights_are_dirty: bool,
    shadow_pass: Pass,
    forward_pass: Pass,
    forward_depth: wgpu::TextureView,
    entity_bind_group: wgpu::BindGroup,
    light_uniform_buf: wgpu::Buffer,
    entity_uniform_buf: wgpu::Buffer,
    window_id: WindowId,
    is_capturing: bool,
    capture_begin_frame: u64,
}

impl Model {
    const MAX_LIGHTS: usize = 10;
    const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: 512,
        height: 512,
        depth: Self::MAX_LIGHTS as u32,
    };
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 20.0);
        let mx_view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(3.0f32, -10.0, 6.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        mx_correction * mx_projection * mx_view
    }
}

fn vertex(pos: [i8; 3], nor: [i8; 3]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1],
        _normal: [nor[0], nor[1], nor[2], 0],
    }
}

fn create_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0, 1]),
        vertex([1, -1, 1], [0, 0, 1]),
        vertex([1, 1, 1], [0, 0, 1]),
        vertex([-1, 1, 1], [0, 0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [0, 0, -1]),
        vertex([1, 1, -1], [0, 0, -1]),
        vertex([1, -1, -1], [0, 0, -1]),
        vertex([-1, -1, -1], [0, 0, -1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [1, 0, 0]),
        vertex([1, 1, -1], [1, 0, 0]),
        vertex([1, 1, 1], [1, 0, 0]),
        vertex([1, -1, 1], [1, 0, 0]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [-1, 0, 0]),
        vertex([-1, 1, 1], [-1, 0, 0]),
        vertex([-1, 1, -1], [-1, 0, 0]),
        vertex([-1, -1, -1], [-1, 0, 0]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [0, 1, 0]),
        vertex([-1, 1, -1], [0, 1, 0]),
        vertex([-1, 1, 1], [0, 1, 0]),
        vertex([1, 1, 1], [0, 1, 0]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, -1, 0]),
        vertex([-1, -1, 1], [0, -1, 0]),
        vertex([-1, -1, -1], [0, -1, 0]),
        vertex([1, -1, -1], [0, -1, 0]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_plane(size: i8) -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        vertex([size, -size, 0], [0, 0, 1]),
        vertex([size, size, 0], [0, 0, 1]),
        vertex([-size, -size, 0], [0, 0, 1]),
        vertex([-size, size, 0], [0, 0, 1]),
    ];

    let index_data: &[u16] = &[0, 1, 2, 2, 1, 3];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_cube_descs() -> Vec<CubeDesc> {
    vec![
        CubeDesc {
            offset: cgmath::vec3(-2.0, -2.0, 2.0),
            angle: 10.0,
            scale: 0.7,
            rotation: 0.1,
        },
        CubeDesc {
            offset: cgmath::vec3(2.0, -2.0, 2.0),
            angle: 50.0,
            scale: 1.3,
            rotation: 0.2,
        },
        CubeDesc {
            offset: cgmath::vec3(-2.0, 2.0, 2.0),
            angle: 140.0,
            scale: 1.1,
            rotation: 0.3,
        },
        CubeDesc {
            offset: cgmath::vec3(2.0, 2.0, 2.0),
            angle: 210.0,
            scale: 0.9,
            rotation: 0.4,
        },
    ]
}

fn create_entities(device: &wgpu::Device, cube_descs: &Vec<CubeDesc>) -> Vec<Entity> {
    // create cubes
    let (cube_vertex_data, cube_index_data) = create_cube();

    let cube_vertex_buf = Rc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        }),
    );

    let cube_index_buf = Rc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Index Buffer"),
            contents: bytemuck::cast_slice(&cube_index_data),
            usage: wgpu::BufferUsage::INDEX,
        }),
    );

    // create plane
    let (plane_vertex_data, plane_index_data) = create_plane(7);

    let plane_vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Plane Vertex Buffer"),
        contents: bytemuck::cast_slice(&plane_vertex_data),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let plane_index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Plane Index Buffer"),
        contents: bytemuck::cast_slice(&plane_index_data),
        usage: wgpu::BufferUsage::INDEX,
    });

    let mut entities = vec![{
        Entity {
            mx_world: cgmath::Matrix4::identity(),
            rotation_speed: 0.0,
            color: wgpu::Color::WHITE,
            vertex_buf: Rc::new(plane_vertex_buf),
            index_buf: Rc::new(plane_index_buf),
            index_count: plane_index_data.len(),
            uniform_offset: 0,
        }
    }];

    for (i, cube) in cube_descs.iter().enumerate() {
        use cgmath::{Decomposed, Deg, InnerSpace, Quaternion, Rotation3};

        let transform = Decomposed {
            disp: cube.offset.clone(),
            rot: Quaternion::from_axis_angle(cube.offset.normalize(), Deg(cube.angle)),
            scale: cube.scale,
        };
        entities.push(Entity {
            mx_world: cgmath::Matrix4::from(transform),
            rotation_speed: cube.rotation,
            color: wgpu::Color::GREEN,
            vertex_buf: Rc::clone(&cube_vertex_buf),
            index_buf: Rc::clone(&cube_index_buf),
            index_count: cube_index_data.len(),
            uniform_offset: ((i + 1) * wgpu::BIND_BUFFER_ALIGNMENT as usize) as _,
        });
    }

    entities
}

fn create_entity_uniform_buffer(
    device: &wgpu::Device,
    cube_descs: &Vec<CubeDesc>,
) -> (wgpu::Buffer, u64) {
    let num_entities = 1 + cube_descs.len() as wgpu::BufferAddress;
    // Note: dynamic offsets also have to be aligned to `BIND_BUFFER_ALIGNMENT`.
    let entity_uniform_size = mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
    assert!(entity_uniform_size <= wgpu::BIND_BUFFER_ALIGNMENT);
    let entity_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: num_entities * wgpu::BIND_BUFFER_ALIGNMENT,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    (entity_uniform_buf, entity_uniform_size)
}

fn create_shadow(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
    // create shadow
    let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("shadow"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });

    // let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
    //     size: Model::SHADOW_SIZE,
    //     mip_level_count: 1,
    //     sample_count: 1,
    //     dimension: wgpu::TextureDimension::D2,
    //     format: Model::SHADOW_FORMAT,
    //     // usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
    //     usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
    //     label: None,
    // });
    // let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
    // let mut shadow_target_views = (0..2)
    //     .map(|i| {
    //         Some(shadow_texture.create_view(&wgpu::TextureViewDescriptor {
    //             label: Some("shadow"),
    //             format: None,
    //             dimension: Some(wgpu::TextureViewDimension::D2),
    //             aspect: wgpu::TextureAspect::All,
    //             base_mip_level: 0,
    //             level_count: None,
    //             base_array_layer: i as u32,
    //             array_layer_count: NonZeroU32::new(1),
    //         }))
    //     })
    //     .collect::<Vec<_>>();

    let shadow_texture = wgpu::TextureBuilder::new()
        .extent(Model::SHADOW_SIZE)
        .mip_level_count(1)
        .sample_count(1)
        .dimension(wgpu::TextureDimension::D2)
        .format(Model::SHADOW_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
        // .label(None)
        .build(&device);

    let shadow_view = shadow_texture.view().build();

    (shadow_texture, shadow_view, shadow_sampler)
}

fn create_lights(shadow_texture: &wgpu::Texture) -> Vec<Light> {
    let mut shadow_target_views = (0..2)
        .map(|i| {
            // Some(shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            //     label: Some("shadow"),
            //     format: None,
            //     dimension: Some(wgpu::TextureViewDimension::D2),
            //     aspect: wgpu::TextureAspect::All,
            //     base_mip_level: 0,
            //     level_count: None,
            //     base_array_layer: i as u32,
            //     array_layer_count: NonZeroU32::new(1),
            // }))
            Some(
                shadow_texture
                    .view()
                    .label("shadow")
                    // .format(None)
                    .dimension(wgpu::TextureViewDimension::D2)
                    .aspect(wgpu::TextureAspect::All)
                    // .base_mip_level(0)
                    .level_count(None)
                    .base_array_layer(i as u32)
                    .array_layer_count(NonZeroU32::new(1))
                    .build(),
            )
        })
        .collect::<Vec<_>>();

    let lights = vec![
        Light {
            pos: cgmath::Point3::new(7.0, -5.0, 10.0),
            color: wgpu::Color {
                r: 0.5,
                g: 1.0,
                b: 0.5,
                a: 1.0,
            },
            fov: 60.0,
            depth: 1.0..20.0,
            target_view: shadow_target_views[0].take().unwrap(),
        },
        Light {
            pos: cgmath::Point3::new(-5.0, 7.0, 10.0),
            color: wgpu::Color {
                r: 1.0,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            fov: 45.0,
            depth: 1.0..20.0,
            target_view: shadow_target_views[1].take().unwrap(),
        },
    ];

    lights
}

fn create_light_uniform_buffer(device: &wgpu::Device) -> (wgpu::Buffer, u64) {
    let light_uniform_size =
        (Model::MAX_LIGHTS * mem::size_of::<LightRaw>()) as wgpu::BufferAddress;
    let light_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: light_uniform_size,
        usage: wgpu::BufferUsage::UNIFORM
            | wgpu::BufferUsage::COPY_SRC
            | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    (light_uniform_buf, light_uniform_size)
}

fn create_entity_bind_group(
    device: &wgpu::Device,
    entity_uniform_buf: &wgpu::Buffer,
    entity_uniform_size: u64,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let entity_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: true,
                    min_binding_size: BufferSize::new(entity_uniform_size),
                },
                count: None,
            }],
            label: None,
        });

    let entity_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &entity_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            // resource: wgpu::BindingResource::Buffer {
            //     buffer: &entity_uniform_buf,
            //     offset: 0,
            //     size: BufferSize::new(entity_uniform_size),
            // },
            resource: wgpu::BindingResource::Buffer(
                entity_uniform_buf.slice(0..entity_uniform_size),
            ),
        }],
        label: None,
    });

    (entity_bind_group_layout, entity_bind_group)
}

fn create_shadow_uniform_buffer(device: &wgpu::Device) -> (wgpu::Buffer, u64) {
    // create shadow uniform buffer
    let shadow_uniform_size = mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
    let shadow_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: shadow_uniform_size,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });
    (shadow_uniform_buf, shadow_uniform_size)
}

fn create_shadow_bind_group(
    device: &wgpu::Device,
    shadow_uniform_buf: &wgpu::Buffer,
    shadow_uniform_size: u64,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    // create shadow bind group layout
    let shadow_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, // global
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: BufferSize::new(shadow_uniform_size),
                },
                count: None,
            }],
        });

    // create shadow bind group
    let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &shadow_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            // resource: uniform_buf.as_entire_binding(),
            resource: wgpu::BindingResource::Buffer(shadow_uniform_buf.slice(..)),
        }],
        label: None,
    });

    (shadow_bind_group_layout, shadow_bind_group)
}

fn create_shadow_pipeline(
    device: &wgpu::Device,
    entity_bind_group_layout: &wgpu::BindGroupLayout,
    shadow_bind_group_layout: &wgpu::BindGroupLayout,
    vb_desc: &wgpu::VertexBufferDescriptor,
) -> wgpu::RenderPipeline {
    // Create the render pipeline
    let vs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/bake.vert.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow"),
        bind_group_layouts: &[&shadow_bind_group_layout, &entity_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow"),
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: None,
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            // polygon_mode: wgpu::PolygonMode::Fill,
            depth_bias: 2, // corresponds to bilinear filtering
            depth_bias_slope_scale: 2.0,
            depth_bias_clamp: 0.0,
            clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[],
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: Model::SHADOW_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilStateDescriptor::default(),
        }),
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

fn create_forward_uniform_buf(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    lights: &Vec<Light>,
) -> wgpu::Buffer {
    // create forward uniform buffer
    let mx_total = Model::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let forward_uniforms = ForwardUniforms {
        proj: *mx_total.as_ref(),
        num_lights: [lights.len() as u32, 0, 0, 0],
    };
    let forward_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&forward_uniforms),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    });

    forward_uniform_buf
}

fn create_forward_bind_group(
    device: &wgpu::Device,
    forward_uniform_buf: &wgpu::Buffer,
    light_uniform_buf: &wgpu::Buffer,
    light_uniform_size: u64,
    shadow_view: &wgpu::TextureView,
    shadow_sampler: &wgpu::Sampler,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    // create forward bind group layout
    let forward_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // global
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: BufferSize::new(mem::size_of::<ForwardUniforms>() as _),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, // lights
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: BufferSize::new(light_uniform_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, // shadow texture
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        // component_type: wgpu::TextureComponentType::DepthComparison,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3, // shadow sampler
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: true },
                    count: None,
                },
            ],
            label: None,
        });

    // create forward bind group
    let forward_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &forward_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                // resource: uniform_buf.as_entire_binding(),
                resource: wgpu::BindingResource::Buffer(forward_uniform_buf.slice(..)),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                // resource: light_uniform_buf.as_entire_binding(),
                resource: wgpu::BindingResource::Buffer(light_uniform_buf.slice(..)),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
        ],
        label: None,
    });

    (forward_bind_group_layout, forward_bind_group)
}

fn create_forward_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    vb_desc: &wgpu::VertexBufferDescriptor,
    forward_bind_group_layout: &wgpu::BindGroupLayout,
    entity_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Create the render pipeline
    let vs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/forward.vert.spv"));
    let fs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/forward.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("main"),
        bind_group_layouts: &[&forward_bind_group_layout, &entity_bind_group_layout],
        push_constant_ranges: &[],
    });

    let forward_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main"),
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
        color_states: &[sc_desc.format.into()],
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: Model::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilStateDescriptor::default(),
        }),
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[vb_desc.clone()],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    forward_pipeline
}

fn create_forward_depth(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::TextureView {
    // let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
    //     size: wgpu::Extent3d {
    //         width: sc_desc.width,
    //         height: sc_desc.height,
    //         depth: 1,
    //     },
    //     mip_level_count: 1,
    //     sample_count: 1,
    //     dimension: wgpu::TextureDimension::D2,
    //     format: Model::DEPTH_FORMAT,
    //     // usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
    //     usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    //     label: None,
    // });
    let depth_texture = wgpu::TextureBuilder::new()
        .extent(wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        })
        .mip_level_count(1)
        .sample_count(1)
        .dimension(wgpu::TextureDimension::D2)
        .format(Model::DEPTH_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
        // .label(None)
        .build(&device);

    let forward_depth = depth_texture.view().build();

    forward_depth
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
        .resized(resized)
        .key_pressed(key_pressed)
        .raw_view(raw_view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(window_id).unwrap(); // window reference
    let device = window.swap_chain_device(); // reference of gpu device
    let sc_desc = window.swap_chain_descriptor();
    let vb_desc = Vertex::desc();

    // entities
    let cube_descs = create_cube_descs();
    let entities = create_entities(&device, &cube_descs);
    let (entity_uniform_buf, entity_uniform_size) =
        create_entity_uniform_buffer(&device, &cube_descs);
    let (entity_bind_group_layout, entity_bind_group) =
        create_entity_bind_group(&device, &entity_uniform_buf, entity_uniform_size);

    // shadow pass
    // in the shadow render pass, shadows are baked into this texture.
    // in the forward render pass, these texture and sampler are binded as uniforms in bind group
    // so automatically shared with forward render pass after shadow render pass finished
    let (shadow_texture, shadow_view, shadow_sampler) = create_shadow(&device, &sc_desc);
    let (shadow_uniform_buf, shadow_uniform_size) = create_shadow_uniform_buffer(&device);
    let (shadow_bind_group_layout, shadow_bind_group) =
        create_shadow_bind_group(&device, &shadow_uniform_buf, shadow_uniform_size);
    let shadow_pipeline = create_shadow_pipeline(
        &device,
        &entity_bind_group_layout,
        &shadow_bind_group_layout,
        &vb_desc,
    );
    let shadow_pass = Pass {
        pipeline: shadow_pipeline,
        bind_group: shadow_bind_group,
        uniform_buf: shadow_uniform_buf,
    };

    // lights
    let lights = create_lights(&shadow_texture);
    let (light_uniform_buf, light_uniform_size) = create_light_uniform_buffer(&device);

    // forward pass
    let forward_uniform_buf = create_forward_uniform_buf(&device, &sc_desc, &lights);
    let (forward_bind_group_layout, forward_bind_group) = create_forward_bind_group(
        &device,
        &forward_uniform_buf,
        &light_uniform_buf,
        light_uniform_size,
        &shadow_view,
        &shadow_sampler,
    );
    let forward_pipeline = create_forward_pipeline(
        &device,
        &sc_desc,
        &vb_desc,
        &forward_bind_group_layout,
        &entity_bind_group_layout,
    );
    let forward_pass = Pass {
        pipeline: forward_pipeline,
        bind_group: forward_bind_group,
        uniform_buf: forward_uniform_buf,
    };

    // depth texture view of forward pass
    let forward_depth = create_forward_depth(&device, &sc_desc);

    let is_capturing = false;
    let capture_begin_frame = 0u64;

    Model {
        entities,
        lights,
        lights_are_dirty: true,
        shadow_pass,
        forward_pass,
        forward_depth,
        light_uniform_buf,
        entity_uniform_buf,
        entity_bind_group,
        window_id,
        is_capturing,
        capture_begin_frame,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    for entity in model.entities.iter_mut() {
        if entity.rotation_speed != 0.0 {
            let rotation = cgmath::Matrix4::from_angle_x(cgmath::Deg(entity.rotation_speed));
            entity.mx_world = entity.mx_world * rotation;
        }
    }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn resized(app: &App, model: &mut Model, _: Vector2) {
    let window = app.window(model.window_id).unwrap();
    let sc_desc = window.swap_chain_descriptor();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();

    // update view-projection matrix
    let mx_total = Model::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    queue.write_buffer(
        &model.forward_pass.uniform_buf,
        0,
        bytemuck::cast_slice(mx_ref),
    );

    // let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
    //     size: wgpu::Extent3d {
    //         width: sc_desc.width,
    //         height: sc_desc.height,
    //         depth: 1,
    //     },
    //     mip_level_count: 1,
    //     sample_count: 1,
    //     dimension: wgpu::TextureDimension::D2,
    //     format: Model::DEPTH_FORMAT,
    //     usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    //     label: None,
    // });
    // model.forward_depth = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_texture = wgpu::TextureBuilder::new()
        .extent(wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        })
        .mip_level_count(1)
        .sample_count(1)
        .dimension(wgpu::TextureDimension::D2)
        .format(Model::DEPTH_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
        // .label(None)
        .build(&device);

    model.forward_depth = depth_texture.view().build();
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

    for entity in model.entities.iter() {
        let data = EntityUniforms {
            model: entity.mx_world.into(),
            color: [
                entity.color.r as f32,
                entity.color.g as f32,
                entity.color.b as f32,
                entity.color.a as f32,
            ],
        };
        queue.write_buffer(
            &model.entity_uniform_buf,
            entity.uniform_offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&data),
        );
    }

    // if model.lights_are_dirty {
    //     model.lights_are_dirty = false;
    for (i, light) in model.lights.iter().enumerate() {
        queue.write_buffer(
            &model.light_uniform_buf,
            (i * mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
            bytemuck::bytes_of(&light.to_raw()),
        );
    }
    // }

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // encoder.push_debug_group("shadow passes");
    for (i, light) in model.lights.iter().enumerate() {
        // encoder.push_debug_group(&format!(
        //     "shadow pass {} (light at position {:?})",
        //     i, light.pos
        // ));

        // The light uniform buffer already has the projection,
        // let's just copy it over to the shadow uniform buffer.
        encoder.copy_buffer_to_buffer(
            &model.light_uniform_buf,
            (i * mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
            &model.shadow_pass.uniform_buf,
            0,
            64,
        );

        // encoder.insert_debug_marker("render entities");
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &light.target_view, // bake to shadow texture
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            pass.set_pipeline(&model.shadow_pass.pipeline);
            pass.set_bind_group(0, &model.shadow_pass.bind_group, &[]);

            for entity in &model.entities {
                pass.set_bind_group(1, &model.entity_bind_group, &[entity.uniform_offset]);
                pass.set_index_buffer(entity.index_buf.slice(..));
                pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
            }
        }

        // encoder.pop_debug_group();
    }
    // encoder.pop_debug_group();

    // forward pass
    // encoder.push_debug_group("forward rendering pass");
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                // attachment: &frame.texture_view(),
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &model.forward_depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });
        pass.set_pipeline(&model.forward_pass.pipeline);
        pass.set_bind_group(0, &model.forward_pass.bind_group, &[]);

        for entity in &model.entities {
            pass.set_bind_group(1, &model.entity_bind_group, &[entity.uniform_offset]);
            pass.set_index_buffer(entity.index_buf.slice(..));
            pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
            pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
        }
    }
    // encoder.pop_debug_group();

    queue.submit(iter::once(encoder.finish()));

    // let mut encoder = raw_frame.command_encoder();
    // encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    // let mut render_pass = wgpu::RenderPassBuilder::new()
    //     .color_attachment(frame.texture_view(), |color| color)
    //     .depth_stencil_attachment(&g.depth_texture_view, |depth| depth)
    //     .begin(&mut encoder);
    // render_pass.set_bind_group(0, &g.bind_group, &[]); // set uniforms
    // render_pass.set_pipeline(&g.render_pipeline);
    // render_pass.set_vertex_buffer(0, &g.vertex_buffer, 0, 0);
    // render_pass.set_vertex_buffer(1, &g.normal_buffer, 0, 0);
    // render_pass.set_index_buffer(&g.index_buffer, 0, 0);

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
