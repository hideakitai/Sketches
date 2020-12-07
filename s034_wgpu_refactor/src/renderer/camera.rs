// TODO: improve based on ofNode, ofCamera and ofEasyCam + WASD
use nannou::math::cgmath::{self, Matrix4, Rad};
use nannou::prelude::*;
use nannou::winit::dpi::LogicalPosition;
use std::clone::Clone;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use super::binding::{Binding, BindingBuilder, BindingType};

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

pub struct CameraRotation {
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

// TODO: enum { Target, Rotation } ??
pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub target: Option<cgmath::Point3<f32>>,
    pub rotation: Option<CameraRotation>,
    pub up: cgmath::Vector3<f32>,
    pub projection: Projection,
    pub raw: UniformTransformRaw,
    pub binding: Binding,
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
        raw.proj_matrix = projection.projection_matrix();

        let binding = BindingBuilder::new()
            .uniform_buffer(
                "camera_view_proj",
                &[raw],
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                false,
            )
            .build(device);

        Self {
            position,
            target: Some(target),
            rotation: None,
            up,
            projection,
            raw,
            binding,
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

        let binding = BindingBuilder::new()
            .uniform_buffer(
                "camera_view_proj",
                &[raw],
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                false,
            )
            .build(device);

        Self {
            position: position.into(),
            target: None,
            rotation: Some(CameraRotation {
                yaw: yaw.into(),
                pitch: pitch.into(),
            }),
            up,
            projection,
            raw,
            binding,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.update_view_proj(device, queue);
    }

    fn update_view_proj(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        self.raw = UniformTransformRaw {
            view_position: self.position.to_homogeneous(),
            view_matrix: self.view_matrix(),
            proj_matrix: self.projection.projection_matrix(),
        };

        // self.binding.write_buffer_at_index(queue, 0, 0, &[self.raw]);
        self.binding
            .write_buffer_at_label(queue, "camera_view_proj", 0, &[self.raw]);
    }

    pub fn view_matrix(&self) -> cgmath::Matrix4<f32> {
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

    pub fn view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.projection.projection_matrix() * self.view_matrix()
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.projection.resized(width, height);
    }
}
