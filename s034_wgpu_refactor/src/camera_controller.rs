use nannou::math::cgmath::{self, Matrix4, Rad};
use nannou::prelude::*;
use nannou::winit::dpi::LogicalPosition;
use std::clone::Clone;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use crate::renderer::camera::{Camera, CameraRotation};

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
