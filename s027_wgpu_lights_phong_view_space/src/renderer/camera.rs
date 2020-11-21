use nannou::math::cgmath;
use nannou::prelude::*;

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

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    //
    pub raw: UniformTransformRaw,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    /// Wgpu's projection space is based on DirectX.
    /// On the other hand, cgmath is based on OpenGL.
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

    pub fn new(
        device: &wgpu::Device,
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut raw = UniformTransformRaw {
            view_position: Zero::zero(),
            view_matrix: cgmath::Matrix4::identity(),
            proj_matrix: cgmath::Matrix4::identity(),
        };
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        raw.view_position = eye.to_homogeneous();
        // self.view_proj = Camera::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix();
        raw.view_matrix = cgmath::Matrix4::look_at(eye, target, up);
        raw.proj_matrix = cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);
        let buffer: wgpu::Buffer = Self::create_uniform_buffer(device, &raw);
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group =
            Self::create_bind_group_from_buffers(device, &bind_group_layout, &[&buffer]);

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
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
        self.raw.view_position = self.eye.to_homogeneous();
        // self.view_proj = Camera::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix();
        self.raw.view_matrix = self.build_view_matrix();
        self.raw.proj_matrix = self.build_proj_matrix();
    }

    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at(self.eye, self.target, self.up)
    }

    pub fn build_proj_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.build_proj_matrix() * self.build_view_matrix()
    }
}

pub struct CameraController {
    speed: f32,
    _is_up_pressed: bool,
    _is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            _is_up_pressed: false,
            _is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    // fn process_events(&mut self, event: &WindowEvent) -> bool {
    //     match event {
    //         WindowEvent::KeyboardInput {
    //             input:
    //                 KeyboardInput {
    //                     state,
    //                     virtual_keycode: Some(keycode),
    //                     ..
    //                 },
    //             ..
    //         } => {
    //             let is_pressed = *state == ElementState::Pressed;
    //             match keycode {
    //                 VirtualKeyCode::Space => {
    //                     self.is_up_pressed = is_pressed;
    //                     true
    //                 }
    //                 VirtualKeyCode::LShift => {
    //                     self.is_down_pressed = is_pressed;
    //                     true
    //                 }
    //                 VirtualKeyCode::W | VirtualKeyCode::Up => {
    //                     self.is_forward_pressed = is_pressed;
    //                     true
    //                 }
    //                 VirtualKeyCode::A | VirtualKeyCode::Left => {
    //                     self.is_left_pressed = is_pressed;
    //                     true
    //                 }
    //                 VirtualKeyCode::S | VirtualKeyCode::Down => {
    //                     self.is_backward_pressed = is_pressed;
    //                     true
    //                 }
    //                 VirtualKeyCode::D | VirtualKeyCode::Right => {
    //                     self.is_right_pressed = is_pressed;
    //                     true
    //                 }
    //                 _ => false,
    //             }
    //         }
    //         _ => false,
    //     }
    // }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
