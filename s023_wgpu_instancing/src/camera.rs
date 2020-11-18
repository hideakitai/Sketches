use nannou::math::cgmath;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
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

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
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
        let forward = (camera.target - camera.eye).normalize();

        if self.is_forward_pressed {
            camera.eye += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * self.speed;
        }

        let right = forward.cross(camera.up);

        if self.is_right_pressed {
            camera.eye += right * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed;
        }
    }
}
