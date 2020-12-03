use nannou::math::cgmath::{self, Matrix4, Rad};
use nannou::prelude::*;
use nannou::winit::dpi::LogicalPosition;
use std::clone::Clone;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use super::buffer::BufferUtil;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformTransformRaw {
    view_position: cgmath::Vector4<f32>,
    view_matrix: cgmath::Matrix4<f32>,
    proj_matrix: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for UniformTransformRaw {}
unsafe impl bytemuck::Zeroable for UniformTransformRaw {}

pub struct Projection {
    pub aspect: f32,
    // pub fovy: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Projection {
    /// Wgpu's projection space is based on DirectX (left hand).
    /// On the other hand, cgmath is based on OpenGL (right hand).
    /// So this matrix convert projection view from OpenGL to DirectX.
    /// e.g. map z axis from [-1.0, 1.0] to [0.0, 1.0]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    #[allow(unused)]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Self::OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

impl BufferUtil for Camera {
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::BindGroupLayoutBuilder::new()
            // .label(Some("uniform_bind_group_layout")),
            .uniform_buffer(
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                false,
            )
            .build(device)
    }
}

pub struct CameraRotation {
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub target: Option<cgmath::Point3<f32>>,
    pub rotation: Option<CameraRotation>,
    pub up: cgmath::Vector3<f32>,
    pub projection: Projection,
    // pub aspect: f32,
    // pub fovy: f32,
    // pub znear: f32,
    // pub zfar: f32,
    //
    pub raw: UniformTransformRaw,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn from_target(
        device: &wgpu::Device,
        position: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        // aspect: f32,
        width: u32,
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut raw = UniformTransformRaw {
            view_position: Zero::zero(),
            view_matrix: cgmath::Matrix4::identity(),
            proj_matrix: cgmath::Matrix4::identity(),
        };
        let projection = Projection::new(width, height, cgmath::Deg(fovy), znear, zfar);
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        raw.view_position = position.to_homogeneous();
        raw.view_matrix = cgmath::Matrix4::look_at(position, target, up);
        // raw.proj_matrix = cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);
        raw.proj_matrix = projection.projection_matrix();
        let buffer: wgpu::Buffer = Self::create_uniform_buffer(device, &raw);
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group =
            Self::create_bind_group_from_buffers(device, &bind_group_layout, &[&buffer]);

        Self {
            position,
            target: Some(target),
            rotation: None,
            up,
            projection,
            // aspect,
            // fovy,
            // znear,
            // zfar,
            raw,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn from_rotation<
        V: Into<cgmath::Point3<f32>> + Clone,
        Y: Into<Rad<f32>> + Clone,
        P: Into<Rad<f32>> + Clone,
    >(
        device: &wgpu::Device,
        position: V,
        yaw: Y,
        pitch: P,
        up: cgmath::Vector3<f32>,
        // aspect: f32,
        width: u32,
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut raw = UniformTransformRaw {
            view_position: Zero::zero(),
            view_matrix: cgmath::Matrix4::identity(),
            proj_matrix: cgmath::Matrix4::identity(),
        };
        let projection = Projection::new(width, height, cgmath::Deg(fovy), znear, zfar);
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        raw.view_position = position.clone().into().to_homogeneous();
        raw.view_matrix = Matrix4::look_at_dir(
            position.clone().into(),
            cgmath::Vector3::new(
                yaw.clone().into().0.cos(),
                pitch.clone().into().0.sin(),
                yaw.clone().into().0.sin(),
            )
            .normalize(),
            up,
        );
        // raw.view_matrix = cgmath::Matrix4::look_at(position, target, up);
        // raw.proj_matrix = cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);
        raw.proj_matrix = projection.projection_matrix();
        let buffer: wgpu::Buffer = Self::create_uniform_buffer(device, &raw);
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group =
            Self::create_bind_group_from_buffers(device, &bind_group_layout, &[&buffer]);

        Self {
            position: position.into(),
            target: None,
            rotation: Some(CameraRotation {
                yaw: yaw.into(),
                pitch: pitch.into(),
            }),
            up,
            projection,
            // aspect,
            // fovy,
            // znear,
            // zfar,
            raw,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.update_view_proj();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.raw]));
    }

    pub fn update_view_proj(&mut self) {
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        self.raw.view_position = self.position.to_homogeneous();
        self.raw.view_matrix = self.view_matrix();
        self.raw.proj_matrix = self.projection.projection_matrix();
    }

    pub fn view_matrix(&self) -> cgmath::Matrix4<f32> {
        // TODO: if there is target
        // TODO: if there is rotation
        // match self.target {
        //     Some(target) => cgmath::Matrix4::look_at(self.position, target, self.up),
        //     None => match self.rotation {
        //         Some(rotation) => Matrix4::look_at_dir(
        //             self.position,
        //             cgmath::Vector3::new(
        //                 rotation.yaw.0.cos(),
        //                 rotation.pitch.0.sin(),
        //                 rotation.yaw.0.sin(),
        //             )
        //             .normalize(),
        //             self.up,
        //         ),
        //         None => panic!("camera must have target or rotation"),
        //     },
        // }

        if let Some(target) = self.target {
            cgmath::Matrix4::look_at(self.position, target, self.up)
        } else if let Some(rotation) = &self.rotation {
            Matrix4::look_at_dir(
                self.position,
                cgmath::Vector3::new(
                    rotation.yaw.0.cos(),
                    rotation.pitch.0.sin(),
                    rotation.yaw.0.sin(),
                )
                .normalize(),
                self.up,
            )
        } else {
            panic!("camera must have target or rotation")
        }
    }

    // pub fn build_proj_matrix(&self) -> cgmath::Matrix4<f32> {
    //     cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    // }

    pub fn view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.projection.projection_matrix() * self.view_matrix()
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.projection.resized(width, height);
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_key(&mut self, key: Key, is_pressed: bool) -> bool {
        let amount = if is_pressed { 1.0 } else { 0.0 };
        match key {
            Key::W | Key::Up => {
                self.amount_forward = amount;
                true
            }
            Key::S | Key::Down => {
                self.amount_backward = amount;
                true
            }
            Key::A | Key::Left => {
                self.amount_left = amount;
                true
            }
            Key::D | Key::Right => {
                self.amount_right = amount;
                true
            }
            Key::Space => {
                self.amount_up = amount;
                true
            }
            Key::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(LogicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        if let Some(rotation) = &mut camera.rotation {
            let pitch = rotation.pitch;
            let yaw = rotation.yaw;

            // Move forward/backward and left/right
            let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
            let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
            let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
            camera.position +=
                forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
            camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

            // Move in/out (aka. "zoom")
            // Note: this isn't an actual zoom. The camera's position
            // changes when zooming. I've added this to make it easier
            // to get closer to an object you want to focus on.
            let (pitch_sin, pitch_cos) = pitch.0.sin_cos();
            let scrollward =
                cgmath::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin)
                    .normalize();
            camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
            self.scroll = 0.0;

            // Move up/down. Since we don't use roll, we can just
            // modify the y coordinate directly.
            camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

            // Rotate
            let yaw = yaw + Rad(self.rotate_horizontal) * self.sensitivity * dt;
            let mut pitch = pitch + Rad(-self.rotate_vertical) * self.sensitivity * dt;
            // camera.rotation.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
            // camera.rotation.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

            // Keep the camera's angle from going too high/low.
            if pitch < -Rad(FRAC_PI_2) {
                pitch = -Rad(FRAC_PI_2);
            } else if pitch > Rad(FRAC_PI_2) {
                pitch = Rad(FRAC_PI_2);
            }

            // If process_mouse isn't called every frame, these values
            // will not get set to zero, and the camera will rotate
            // when moving in a non cardinal direction.
            self.rotate_horizontal = 0.0;
            self.rotate_vertical = 0.0;

            camera.rotation = Some(CameraRotation { yaw, pitch });
        } else {
            panic!("camera should have rotation")
        }
    }
}
