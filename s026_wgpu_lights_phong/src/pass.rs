use crate::camera::{Camera, CameraController};
use crate::texture::Texture;
// use crate::vertex::Vertex;
use crate::geom;
use crate::geom::{DrawGeom, GeomVertex, Vertex};
use crate::light::{DrawLight, Light};
use nannou::math::cgmath;
use nannou::prelude::*;

pub struct Pass {
    obj_model: geom::Geom,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    // num_indices: usize,
    // // _diffuse_texture: texture::Texture,
    // diffuse_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    // _instance_buffer: wgpu::Buffer,
    camera: Camera,
    camera_controller: CameraController,
    light: Light,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    render_pipeline: wgpu::RenderPipeline,
}

impl Pass {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;
    // const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    //     Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    //     0.0,
    //     Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    // );

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        // let (vertex_buffer, index_buffer, num_indices) = create_objects(device);
        let (instances, instance_buffer) = create_instances(device);

        // let diffuse_texture = create_texture(device, queue);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);
        // let diffuse_bind_group =
        //     create_diffuse_bind_group(device, &texture_bind_group_layout, &diffuse_texture);

        let res_dir = std::path::Path::new("..").join("assets").join("learn_wgpu");
        let obj_model = geom::Geom::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            res_dir.join("cube.obj"),
        )
        .unwrap();

        let depth_texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");

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

        let light = Light {
            position: (2.0, 2.0, 2.0).into(),
            _padding: 0,
            color: (1.0, 1.0, 1.0).into(),
        };
        let light_buffer = create_light_uniform_buffer(device, &light);
        let light_bind_group_layout = create_light_bind_group_layout(device);
        let light_bind_group =
            create_light_bind_group(device, &light_buffer, &light_bind_group_layout);

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
            &light_bind_group_layout,
        );
        let render_pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            sc_desc.format,
            Some(Texture::DEPTH_FORMAT),
            &[GeomVertex::desc()],
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv")),
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv")),
        );

        let light_pipeline_layout = create_light_render_pipeline_layout(
            device,
            &uniform_bind_group_layout,
            &light_bind_group_layout,
        );
        let light_render_pipeline = {
            create_render_pipeline(
                &device,
                &light_pipeline_layout,
                sc_desc.format,
                Some(Texture::DEPTH_FORMAT),
                &[GeomVertex::desc()],
                wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.vert.spv")),
                wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.frag.spv")),
            )
        };

        Self {
            obj_model,
            instances,
            camera,
            camera_controller,
            light,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            render_pipeline,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        // update camera
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // update the light
        let old_position = self.light.position;
        self.light.position =
            cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position;
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        // render_pass.set_pipeline(&self.render_pipeline);
        // // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        // // render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        // // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // // render_pass.set_index_buffer(self.index_buffer.slice(..));
        // // render_pass.draw_indexed(0..self.num_indices as u32, 0, 0..self.instances.len() as _);
        // render_pass.draw_geom_instanced(
        //     &self.obj_model,
        //     0..self.instances.len() as u32,
        //     &self.uniform_bind_group,
        // );
        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.draw_light_model(
            &self.obj_model,
            &self.uniform_bind_group,
            &self.light_bind_group,
        );
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_geom_instanced(
            &self.obj_model,
            0..self.instances.len() as u32,
            &self.uniform_bind_group,
            &self.light_bind_group,
        );
    }

    pub fn resized(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.camera.aspect = sc_desc.width as f32 / sc_desc.height as f32;
        self.depth_texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");
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
    view_position: cgmath::Vector4<f32>,
    view_proj: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_position: Zero::zero(),
            view_proj: cgmath::Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        self.view_position = camera.eye.to_homogeneous();
        // self.view_proj = Camera::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix();
        self.view_proj = camera.build_view_projection_matrix();
    }
}

fn create_instances(device: &wgpu::Device) -> (Vec<Instance>, wgpu::Buffer) {
    let instances = (0..Pass::NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..Pass::NUM_INSTANCES_PER_ROW).map(move |x| {
                // let position = cgmath::Vector3 {
                //     x: x as f32,
                //     y: 0.0,
                //     z: z as f32,
                // } - Pass::INSTANCE_DISPLACEMENT;
                let x = Pass::SPACE_BETWEEN * (x as f32 - Pass::NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = Pass::SPACE_BETWEEN * (z as f32 - Pass::NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let position = cgmath::Vector3 { x, y: 0.0, z };
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

pub fn create_light_uniform_buffer(device: &wgpu::Device, light: &Light) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light VB"),
        contents: bytemuck::cast_slice(&[*light]),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    })
}

pub fn create_light_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: None,
    })
}

pub fn create_light_bind_group(
    device: &wgpu::Device,
    light_buffer: &wgpu::Buffer,
    light_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(light_buffer.slice(..)),
        }],
        label: None,
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
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
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
    light_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            texture_bind_group_layout,
            uniform_bind_group_layout,
            light_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}

pub fn create_light_render_pipeline_layout(
    device: &wgpu::Device,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    light_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Light Pipeline Layout"),
        bind_group_layouts: &[&uniform_bind_group_layout, &light_bind_group_layout],
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
    // vs_src: wgpu::ShaderModuleSource,
    // fs_src: wgpu::ShaderModuleSource,
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    // let vs_module = device.create_shader_module(vs_src);
    // let fs_module = device.create_shader_module(fs_src);

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

// pub fn create_render_pipeline(
//     device: &wgpu::Device,
//     sc_desc: &wgpu::SwapChainDescriptor,
//     render_pipeline_layout: &wgpu::PipelineLayout,
// ) -> wgpu::RenderPipeline {
//     // let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
//     // let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));
//     let vs_module =
//         wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv"));
//     let fs_module =
//         wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv"));

//     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some("Render Pipeline"),
//         layout: Some(&render_pipeline_layout),
//         vertex_stage: wgpu::ProgrammableStageDescriptor {
//             module: &vs_module,
//             entry_point: "main",
//         },
//         fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
//             module: &fs_module,
//             entry_point: "main",
//         }),
//         rasterization_state: Some(wgpu::RasterizationStateDescriptor {
//             front_face: wgpu::FrontFace::Ccw,
//             cull_mode: wgpu::CullMode::Back,
//             depth_bias: 0,
//             depth_bias_slope_scale: 0.0,
//             depth_bias_clamp: 0.0,
//             clamp_depth: false,
//         }),
//         primitive_topology: wgpu::PrimitiveTopology::TriangleList,
//         color_states: &[wgpu::ColorStateDescriptor {
//             format: sc_desc.format,
//             color_blend: wgpu::BlendDescriptor::REPLACE,
//             alpha_blend: wgpu::BlendDescriptor::REPLACE,
//             write_mask: wgpu::ColorWrite::ALL,
//         }],
//         depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
//             format: Texture::DEPTH_FORMAT,
//             depth_write_enabled: true,
//             depth_compare: wgpu::CompareFunction::Less,
//             stencil: wgpu::StencilStateDescriptor::default(),
//         }),
//         vertex_state: wgpu::VertexStateDescriptor {
//             index_format: wgpu::IndexFormat::Uint32,
//             vertex_buffers: &[crate::geom::GeomVertex::desc()],
//         },
//         sample_count: 1,
//         sample_mask: !0,
//         alpha_to_coverage_enabled: false,
//     })
// }
