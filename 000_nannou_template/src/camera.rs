use nannou::math::cgmath::{self, Matrix3, Matrix4, Point3, Rad, Vector3};
use nannou::prelude::*;

// A simple first person camera.
pub struct Camera {
    pub eye: Point3<f32>, // The position of the camera.
    pub pitch: f32,       // Rotation around the x axis.
    pub yaw: f32,         // Rotation around the y axis.
}

impl Camera {
    // Calculate the direction vector from the pitch and yaw.
    pub fn direction(&self) -> Vector3<f32> {
        pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    // The camera's "view" matrix.
    pub fn view(&self) -> Matrix4<f32> {
        let direction = self.direction();
        let up = Vector3::new(0.0, 1.0, 0.0);
        Matrix4::look_at_dir(self.eye, direction, up)
    }
}

pub fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vector3<f32> {
    let xz_unit_len = pitch.cos();
    let x = xz_unit_len * yaw.cos();
    let y = pitch.sin();
    let z = xz_unit_len * (-yaw).sin();
    Vector3::new(x, y, z)
}
