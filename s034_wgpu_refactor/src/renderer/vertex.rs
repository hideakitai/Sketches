use bytemuck::{Pod, Zeroable};
use nannou::math::cgmath;
use nannou::prelude::*;

pub trait VertexDescription {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

// TODO: add color
// TODO: how to handle no content field? Option<T>? wired... or separate elements?
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: cgmath::Vector3<f32>,
    tex_coords: cgmath::Vector2<f32>,
    normal: cgmath::Vector3<f32>,
    tangent: cgmath::Vector3<f32>,
    bitangent: cgmath::Vector3<f32>,
}
unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    pub fn new(
        vertex: [f32; 3],
        tex_coords: [f32; 2],
        normal: [f32; 3],
        tangent: [f32; 3],
        bitangent: [f32; 3],
    ) -> Self {
        Self {
            position: vertex.into(),
            tex_coords: tex_coords.into(),
            normal: normal.into(),
            tangent: tangent.into(),
            bitangent: bitangent.into(),
        }
    }
}

impl VertexDescription for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                // tex_coords
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                // normal
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                // tangent
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                // bitangent
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}
