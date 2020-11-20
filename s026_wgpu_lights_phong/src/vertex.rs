use bytemuck::{Pod, Zeroable};
use nannou::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 3], tc: [f32; 2]) -> Self {
        Self {
            position: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
            tex_coords: [tc[0] as f32, tc[1] as f32],
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        let vertex_size = std::mem::size_of::<Vertex>();
        wgpu::VertexBufferDescriptor {
            stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}
