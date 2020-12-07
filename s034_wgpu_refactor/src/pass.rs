// use buffer::{Bindable, UniformBindable};
use nannou::math::cgmath;
use nannou::prelude::*;
use rayon::prelude::*;
use std::time::Duration;

use crate::camera_controller::CameraController;
use crate::pass_compute::PassCompute;

use crate::renderer::{
    self,
    camera::Camera,
    geom::{DrawGeom, Geom},
    instance::{Instance, Instances},
    light::{DrawLight, Light},
    texture::TextureSet,
    vertex::{Vertex, VertexDescription},
};

pub struct PassMain {
    obj_model: Geom,
    instances: Instances,
    camera: Camera,
    camera_controller: CameraController,
    light: Light,
    light_render_pipeline: wgpu::RenderPipeline,
    depth_texture: TextureSet,
    render_pipeline: wgpu::RenderPipeline,
}

impl PassMain {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let instances = Self::create_instances(device);

        let dir = std::path::Path::new("..").join("assets").join("learn_wgpu");
        let obj_model = Geom::load(&device, &queue, dir.join("cube.obj")).unwrap();

        obj_model
            .meshes
            .par_iter()
            .map(|m| {
                let compute_pass = PassCompute::new(device, &m.vertices, &m.indices, &m.binding);
                compute_pass.render(device, queue)
            })
            .collect::<Vec<Result<(), ()>>>();

        // let camera = Camera::from_target(
        //     device,
        //     (0.0, 5.0, -10.0).into(),
        //     (0.0, 0.0, 0.0).into(),
        //     cgmath::Vector3::unit_y(),
        //     sc_desc.width,
        //     sc_desc.height,
        //     45.0,
        //     0.1,
        //     100.0,
        // );
        let camera = Camera::from_rotation(
            device,
            cgmath::Point3::new(0.0, 5.0, 10.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
            cgmath::Vector3::unit_y(),
            sc_desc.width,
            sc_desc.height,
            45.0,
            0.1,
            100.0,
        );
        let camera_controller = CameraController::new(4.0, 0.4);

        let light = Light::new(device, (2.0, 2.0, 2.0).into(), (1.0, 1.0, 1.0).into());

        let vs_mod =
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv"));
        let fs_mod =
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv"));
        let render_pipeline_layout = renderer::PipelineLayoutBuilder::new()
            .bind_group_layouts(&[
                &camera.binding.bind_group_layout(),
                &instances.binding.bind_group_layout(),
                &light.binding.bind_group_layout(),
                &obj_model.materials[0].binding.bind_group_layout(), // TODO:
            ])
            .build(device);

        let render_pipeline =
            wgpu::RenderPipelineBuilder::from_layout(&render_pipeline_layout, &vs_mod)
                // .label(Some("Render Pipeline")) // TODO:
                .fragment_shader(&fs_mod)
                // .front_face(wgpu::FrontFace::Ccw)
                .cull_mode(wgpu::CullMode::Back)
                // .depth_bias(0)
                // .depth_bias_slope_scale(0.0)
                // .depth_bias_clamp(0.0)
                // .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
                .color_format(sc_desc.format)
                .color_blend(wgpu::BlendDescriptor::REPLACE)
                .alpha_blend(wgpu::BlendDescriptor::REPLACE)
                // .write_mask(wgpu::ColorWrite::ALL)
                .depth_format(Self::DEPTH_FORMAT)
                // .depth_write_enabled(true)
                .depth_compare(wgpu::CompareFunction::Less)
                // .stencil_front(stencil)
                // .stencil_back(stencil)
                // .stencil_read_mask(mask)
                // .stencil_write_mask(mask)
                .sample_count(1)
                // .sample_mask(!0)
                // .index_format(wgpu::IndexFormat::Uint32)
                .add_vertex_buffer_descriptor(Vertex::desc())
                .build(device);

        let vs_mod =
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.vert.spv"));
        let fs_mod =
            wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.frag.spv"));
        let light_pipeline_layout = renderer::PipelineLayoutBuilder::new()
            .bind_group_layouts(&[
                &camera.binding.bind_group_layout(),
                &light.binding.bind_group_layout(),
            ])
            .build(device);

        let light_render_pipeline =
            wgpu::RenderPipelineBuilder::from_layout(&light_pipeline_layout, &vs_mod)
                // .label(Some("Render Pipeline")) // TODO:
                .fragment_shader(&fs_mod)
                // .front_face(wgpu::FrontFace::Ccw)
                .cull_mode(wgpu::CullMode::Back)
                // .depth_bias(0)
                // .depth_bias_slope_scale(0.0)
                // .depth_bias_clamp(0.0)
                // .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
                .color_format(sc_desc.format)
                .color_blend(wgpu::BlendDescriptor::REPLACE)
                .alpha_blend(wgpu::BlendDescriptor::REPLACE)
                // .write_mask(wgpu::ColorWrite::ALL)
                .depth_format(Self::DEPTH_FORMAT)
                // .depth_write_enabled(true)
                .depth_compare(wgpu::CompareFunction::Less)
                // .stencil_front(stencil)
                // .stencil_back(stencil)
                // .stencil_read_mask(mask)
                // .stencil_write_mask(mask)
                .sample_count(1)
                // .sample_mask(!0)
                // .index_format(wgpu::IndexFormat::Uint32)
                .add_vertex_buffer_descriptor(Vertex::desc())
                .build(device);

        let depth_texture = Self::create_depth_texture(device, sc_desc, "depth_texture");

        Self {
            obj_model,
            instances,
            camera,
            camera_controller,
            light,
            light_render_pipeline,
            depth_texture,
            render_pipeline,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dt: Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update(device, queue);

        let old_position = self.light.position().to_owned();
        let new_position = self.light.position_as_mut();
        *new_position = cgmath::Quaternion::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            cgmath::Deg(60.0 * dt.as_secs_f32()),
        ) * old_position;
        self.light.update(queue, dt);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, raw_frame: &wgpu::TextureViewHandle) {
        let camera_bind_group = self.camera.binding.bind_group();
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(raw_frame, |color| {
                color
                    // TODO:
                    .resolve_target(None)
                    // TODO:
                    .load_op(wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }))
                    // TODO:
                    .store_op(true)
            })
            .depth_stencil_attachment(&self.depth_texture.view, |depth| {
                depth
                    .depth_load_op(wgpu::LoadOp::Clear(1.0))
                    .depth_store_op(true)
            })
            .begin(encoder);

        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.draw_light_model(
            &self.obj_model,
            &camera_bind_group,
            &self.light.binding.bind_group(),
        );
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_geom_instanced_with_light_and_inner_material(
            &self.obj_model,
            &camera_bind_group,
            &self.light.binding.bind_group(),
            &self.instances.binding.bind_group(),
            0..self.instances.instances.len() as u32,
        );

        // TODO: make renderer based on nannou's way
        // let device = window.swap_chain_device();
        // let frame_dims: [u32; 2] = window.tracked_state.physical_size.into();
        // let scale_factor = window.tracked_state.scale_factor as f32;
        // let msaa_samples = window.msaa_samples();
        // let target_format = crate::frame::Frame::TEXTURE_FORMAT;
        // let renderer = draw::RendererBuilder::new().build(
        //     device,
        //     frame_dims,
        //     scale_factor,
        //     msaa_samples,
        //     target_format,
        // );
        // draw.to_raw_frame(app, &renderer, &frame).unwrap();
    }

    pub fn resized(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.camera.resized(sc_desc.width, sc_desc.height);
        self.depth_texture = Self::create_depth_texture(device, sc_desc, "depth_texture");
    }

    pub fn key_pressed(&mut self, key: Key) {
        self.camera_controller.process_key(key, true);
    }

    pub fn key_released(&mut self, key: Key) {
        self.camera_controller.process_key(key, false);
    }

    pub fn mouse_moved(&mut self, curr_pos: Point2, prev_pos: Point2) {
        let diff: Vector2<f32> = curr_pos - prev_pos;
        self.camera_controller
            .process_mouse(diff.x as f64, diff.y as f64);
    }

    pub fn mouse_wheel(&mut self, delta: &MouseScrollDelta) {
        self.camera_controller.process_scroll(delta);
    }

    fn create_instances(device: &wgpu::Device) -> Instances {
        let instances = (0..PassMain::NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..PassMain::NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x =
                        Self::SPACE_BETWEEN * (x as f32 - Self::NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z =
                        Self::SPACE_BETWEEN * (z as f32 - Self::NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = cgmath::Vector3 { x, y: 0.0, z };
                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
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

        Instances::from_vec(device, &instances)
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        _label: &str,
    ) -> TextureSet {
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let texture = wgpu::TextureBuilder::new()
            // .label(label)
            .extent(size)
            .mip_level_count(1)
            .sample_count(1)
            .dimension(wgpu::TextureDimension::D2)
            .format(Self::DEPTH_FORMAT)
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        let view = texture.view().build();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        TextureSet {
            texture,
            view,
            sampler,
        }
    }

    // pub fn output_texture(&self) -> &wgpu::Texuture {
    //     self.texture
    // }
}

// // App
// pub fn to_frame(&self, app: &App, frame: &Frame) -> Result<(), draw::renderer::DrawError> {
//     let window_id = frame.window_id();
//     let window = app
//         .window(window_id)
//         .expect("no window to draw to for `Draw`'s window_id");

//     // Retrieve a renderer for this window.
//     let renderers = app.draw_state.renderers.borrow_mut();
//     let renderer = RefMut::map(renderers, |renderers| {
//         renderers.entry(window_id).or_insert_with(|| {
//             let device = window.swap_chain_device();
//             let frame_dims: [u32; 2] = window.tracked_state.physical_size.into();
//             let scale_factor = window.tracked_state.scale_factor as f32;
//             let msaa_samples = window.msaa_samples();
//             let target_format = crate::frame::Frame::TEXTURE_FORMAT;
//             let renderer = draw::RendererBuilder::new().build(
//                 device,
//                 frame_dims,
//                 scale_factor,
//                 msaa_samples,
//                 target_format,
//             );
//             RefCell::new(renderer)
//         })
//     });

//     let scale_factor = window.tracked_state.scale_factor as _;
//     let mut renderer = renderer.borrow_mut();
//     renderer.render_to_frame(window.swap_chain_device(), self, scale_factor, frame);
//     Ok(())
// }
// // Renderer
// pub fn render_to_frame(
//     &mut self,
//     device: &wgpu::Device,
//     draw: &draw::Draw,
//     scale_factor: f32,
//     frame: &Frame,
// ) {
//     let size = frame.texture().size();
//     let attachment = frame.texture_view();
//     let resolve_target = None;
//     let mut command_encoder = frame.command_encoder();
//     self.encode_render_pass(
//         device,
//         &mut *command_encoder,
//         draw,
//         scale_factor,
//         size,
//         attachment,
//         resolve_target,
//     );
// }
// // Renderer
// pub fn encode_render_pass(
//     &mut self,
//     device: &wgpu::Device,
//     encoder: &mut wgpu::CommandEncoder,
//     draw: &draw::Draw,
//     scale_factor: f32,
//     output_attachment_size: [u32; 2],
//     output_attachment: &wgpu::TextureView,
//     resolve_target: Option<&wgpu::TextureView>,
// ) {
//     self.clear();
//     self.fill(device, draw, scale_factor, output_attachment_size);

//     let Renderer {
//         ref pipelines,
//         ref glyph_cache,
//         ref glyph_cache_texture,
//         ref mut depth_texture,
//         ref mut depth_texture_view,
//         ref uniform_bind_group,
//         ref text_bind_group,
//         ref texture_bind_groups,
//         ref mesh,
//         ref vertex_mode_buffer,
//         ref mut render_commands,
//         ref uniform_buffer,
//         scale_factor: ref mut old_scale_factor,
//         ..
//     } = *self;

//     // Update glyph cache texture if necessary.
//     if glyph_cache.requires_upload {
//         glyph_cache_texture.upload_data(device, encoder, &glyph_cache.pixel_buffer);
//     }

//     // Resize the depth texture if the output attachment size has changed.
//     let depth_size = depth_texture.size();
//     if output_attachment_size != depth_size {
//         let depth_format = depth_texture.format();
//         let sample_count = depth_texture.sample_count();
//         *depth_texture =
//             create_depth_texture(device, output_attachment_size, depth_format, sample_count);
//         *depth_texture_view = depth_texture.view().build();
//     }

//     // Retrieve the clear values based on the bg color.
//     let bg_color = draw.state.borrow().background_color;
//     let load_op = match bg_color {
//         None => wgpu::LoadOp::Load,
//         Some(color) => {
//             let (r, g, b, a) = color.into();
//             let (r, g, b, a) = (r as f64, g as f64, b as f64, a as f64);
//             let clear_color = wgpu::Color { r, g, b, a };
//             wgpu::LoadOp::Clear(clear_color)
//         }
//     };

//     // Create render pass builder.
//     let render_pass_builder = wgpu::RenderPassBuilder::new()
//         .color_attachment(output_attachment, |color| {
//             color.resolve_target(resolve_target).load_op(load_op)
//         })
//         .depth_stencil_attachment(&*depth_texture_view, |depth| depth);

//     // Guard for empty mesh.
//     if mesh.points().is_empty() {
//         // Encode the render pass. Only clears the frame.
//         render_pass_builder.begin(encoder);
//         return;
//     }

//     // Create the vertex and index buffers.
//     let vertex_usage = wgpu::BufferUsage::VERTEX;
//     let points_bytes = points_as_bytes(mesh.points());
//     let colors_bytes = colors_as_bytes(mesh.colors());
//     let tex_coords_bytes = tex_coords_as_bytes(mesh.tex_coords());
//     let modes_bytes = vertex_modes_as_bytes(vertex_mode_buffer);
//     let indices_bytes = indices_as_bytes(mesh.indices());
//     let point_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("nannou Renderer point_buffer"),
//         contents: points_bytes,
//         usage: vertex_usage,
//     });
//     let color_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("nannou Renderer color_buffer"),
//         contents: colors_bytes,
//         usage: vertex_usage,
//     });
//     let tex_coords_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("nannou Renderer tex_coords_buffer"),
//         contents: tex_coords_bytes,
//         usage: vertex_usage,
//     });
//     let mode_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("nannou Renderer mode_buffer"),
//         contents: modes_bytes,
//         usage: vertex_usage,
//     });
//     let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("nannou Renderer index_buffer"),
//         contents: indices_bytes,
//         usage: wgpu::BufferUsage::INDEX,
//     });

//     // If the scale factor or window size has changed, update the uniforms for vertex scaling.
//     if *old_scale_factor != scale_factor || output_attachment_size != depth_size {
//         *old_scale_factor = scale_factor;
//         // Upload uniform data for vertex scaling.
//         let uniforms = create_uniforms(output_attachment_size, scale_factor);
//         let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
//         let uniforms_bytes = uniforms_as_bytes(&uniforms);
//         let usage = wgpu::BufferUsage::COPY_SRC;
//         let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
//             label: Some("nannou Renderer uniform_buffer"),
//             contents: uniforms_bytes,
//             usage,
//         });
//         // Copy new uniform buffer state.
//         encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, uniform_buffer, 0, uniforms_size);
//     }

//     // Encode the render pass.
//     let mut render_pass = render_pass_builder.begin(encoder);

//     // Set the buffers.
//     render_pass.set_index_buffer(index_buffer.slice(..));
//     render_pass.set_vertex_buffer(0, point_buffer.slice(..));
//     render_pass.set_vertex_buffer(1, color_buffer.slice(..));
//     render_pass.set_vertex_buffer(2, tex_coords_buffer.slice(..));
//     render_pass.set_vertex_buffer(3, mode_buffer.slice(..));

//     // Set the uniform and text bind groups here.
//     render_pass.set_bind_group(0, uniform_bind_group, &[]);
//     render_pass.set_bind_group(1, text_bind_group, &[]);

//     // Follow the render commands.
//     for cmd in render_commands.drain(..) {
//         match cmd {
//             RenderCommand::SetPipeline(id) => {
//                 let pipeline = &pipelines[&id];
//                 render_pass.set_pipeline(pipeline);
//             }

//             RenderCommand::SetBindGroup(tex_view_id) => {
//                 let bind_group = &texture_bind_groups[&tex_view_id];
//                 render_pass.set_bind_group(2, bind_group, &[]);
//             }

//             RenderCommand::SetScissor(Scissor {
//                 left,
//                 bottom,
//                 width,
//                 height,
//             }) => {
//                 render_pass.set_scissor_rect(left, bottom, width, height);
//             }

//             RenderCommand::DrawIndexed {
//                 start_vertex,
//                 index_range,
//             } => {
//                 let instance_range = 0..1u32;
//                 render_pass.draw_indexed(index_range, start_vertex, instance_range);
//             }
//         }
//     }
// }
