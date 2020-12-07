use nannou::math::cgmath;
use nannou::prelude::*;

use super::binding::Binding;
use super::vertex::Vertex;

pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_id: usize,
    pub binding: Binding,
}

impl Mesh {}
