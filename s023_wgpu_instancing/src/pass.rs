use crate::camera::{Camera, CameraController};
use crate::texture::Texture;
use crate::vertex::Vertex;
use nannou::math::cgmath;
use nannou::prelude::*;

pub struct Pass {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: usize,
    instances: Vec<Instance>,
    // _instance_buffer: wgpu::Buffer,
    // _diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Pass {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let (vertex_buffer, index_buffer, num_indices) = create_objects(device);
        let (instances, instance_buffer) = create_instances(device);

        let diffuse_texture = create_texture(device, queue);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);
        let diffuse_bind_group =
            create_diffuse_bind_group(device, &texture_bind_group_layout, &diffuse_texture);

        let camera = Camera {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(0.2);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);
        let uniform_buffer = create_uniform_buffer(device, &uniforms);
        let uniform_bind_group_layout = create_uniform_bind_group_layout(device);
        let uniform_bind_group = create_uniform_bind_group(
            device,
            &uniform_bind_group_layout,
            &uniform_buffer,
            &instance_buffer,
        );

        let render_pipeline_layout = create_render_pipeline_layout(
            device,
            &texture_bind_group_layout,
            &uniform_bind_group_layout,
        );
        let render_pipeline = create_render_pipeline(device, &sc_desc, &render_pipeline_layout);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            instances,
            // _instance_buffer: instance_buffer,
            // _diffuse_texture: diffuse_texture,
            diffuse_bind_group,
            camera,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, raw_frame: &wgpu::TextureViewHandle) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: raw_frame,
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

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..));
        render_pass.draw_indexed(0..self.num_indices as u32, 0, 0..self.instances.len() as _);
    }

    pub fn aspect(&mut self, aspect: f32) {
        self.camera.aspect = aspect;
    }
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let translation = cgmath::Matrix4::from_translation(self.position);
        let rotation = cgmath::Matrix4::from(self.rotation);
        InstanceRaw {
            model: translation * rotation,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    view_proj: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = Camera::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix();
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>, usize) {
    let vertex_data = [
        Vertex::new(
            [-0.0868241, -0.49240386, 0.0],
            [1.0 - 0.4131759, 1.0 - 0.00759614],
        ), // A
        Vertex::new(
            [-0.49513406, -0.06958647, 0.0],
            [1.0 - 0.0048659444, 1.0 - 0.43041354],
        ), // B
        Vertex::new(
            [-0.21918549, 0.44939706, 0.0],
            [1.0 - 0.28081453, 1.0 - 0.949397057],
        ), // C
        Vertex::new(
            [0.35966998, 0.3473291, 0.0],
            [1.0 - 0.85967, 1.0 - 0.84732911],
        ), // D
        Vertex::new(
            [0.44147372, -0.2347359, 0.0],
            [1.0 - 0.9414737, 1.0 - 0.2652641],
        ), // E
    ];

    let index_data: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

    (vertex_data.to_vec(), index_data.to_vec(), index_data.len())
}

pub fn create_objects(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, usize) {
    let (vertices, indices, num_indices) = create_vertices();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsage::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsage::INDEX,
    });

    (vertex_buffer, index_buffer, num_indices)
}

fn create_instances(device: &wgpu::Device) -> (Vec<Instance>, wgpu::Buffer) {
    let instances = (0..Pass::NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..Pass::NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: x as f32,
                    y: 0.0,
                    z: z as f32,
                } - Pass::INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(
                        position.clone().normalize(),
                        cgmath::Deg(45.0),
                    )
                };

                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();

    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsage::STORAGE,
    });

    (instances, instance_buffer)
}

pub fn create_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
    let diffuse_bytes = include_bytes!("../../assets/learn_wgpu/happy-tree.png");
    Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap()
}

pub fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Uint,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })
}

pub fn create_diffuse_bind_group(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    diffuse_texture: &Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    })
}

pub fn create_uniform_buffer(device: &wgpu::Device, uniforms: &Uniforms) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[*uniforms]),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    })
}

pub fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("uniform_bind_group_layout"),
    })
}

pub fn create_uniform_bind_group(
    device: &wgpu::Device,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    instance_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(instance_buffer.slice(..)),
            },
        ],
        label: Some("uniform_bind_group"),
    })
}

pub fn create_render_pipeline_layout(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[texture_bind_group_layout, uniform_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    render_pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    // let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    // let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));
    let vs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv"));
    let fs_module =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
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
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[Vertex::desc()],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}
