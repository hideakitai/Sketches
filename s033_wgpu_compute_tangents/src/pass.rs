use nannou::math::cgmath;
use nannou::prelude::*;
use std::time::Duration;

use crate::renderer::{
    self, buffer,
    camera::{Camera, CameraController},
    geom::{DrawGeom, Geom, GeomLoader, GeomVertex, Vertex},
    instance::{Instance, Instances},
    light::{DrawLight, Light},
    texture::Texture,
};

pub struct PassMain {
    obj_model: Geom,
    instances: Instances,
    camera: Camera,
    camera_controller: CameraController,
    light: Light,
    light_render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    render_pipeline: wgpu::RenderPipeline,
}

impl PassMain {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let instances = Self::create_instances(device);

        let dir = std::path::Path::new("..").join("assets").join("learn_wgpu");
        let obj_model = GeomLoader::load(&device, &queue, dir.join("cube.obj")).unwrap();

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

        let render_pipeline_layout = renderer::create_render_pipeline_layout(
            device,
            &[
                &camera.bind_group_layout,
                &instances.bind_group_layout,
                &light.bind_group_layout,
                &obj_model.bind_group_layout,
            ],
        );
        let render_pipeline = renderer::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            sc_desc.format,
            Some(Texture::DEPTH_FORMAT),
            &[GeomVertex::desc()],
            &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv")),
            &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv")),
        );

        let light_pipeline_layout = renderer::create_render_pipeline_layout(
            device,
            &[&camera.bind_group_layout, &light.bind_group_layout],
        );
        let light_render_pipeline = {
            renderer::create_render_pipeline(
                &device,
                &light_pipeline_layout,
                sc_desc.format,
                Some(Texture::DEPTH_FORMAT),
                &[GeomVertex::desc()],
                &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.vert.spv")),
                &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/light.frag.spv")),
            )
        };

        let depth_texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");

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

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update(queue);
        self.light.update(queue, dt);
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, raw_frame: &wgpu::TextureViewHandle) {
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
            &self.camera.bind_group,
            &self.light.bind_group,
        );
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_geom_instanced_with_light_and_inner_material(
            &self.obj_model,
            &self.camera.bind_group,
            &self.light.bind_group,
            &self.instances.bind_group,
            0..self.instances.instances.len() as u32,
        );
    }

    pub fn resized(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.camera.resized(sc_desc.width, sc_desc.height);
        // self.camera.aspect = sc_desc.width as f32 / sc_desc.height as f32;
        self.depth_texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");
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
}
