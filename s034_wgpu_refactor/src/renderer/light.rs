// TODO: improve based on ofLight
use nannou::math::cgmath;
use nannou::prelude::*;
use std::ops::Range;
use std::time::Duration;

use super::binding::{Binding, BindingBuilder, BindingType};
use super::geom::Geom;
use super::mesh::Mesh;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightRaw {
    pub position: cgmath::Vector3<f32>,
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32,
    pub color: cgmath::Vector3<f32>,
}
unsafe impl bytemuck::Zeroable for LightRaw {}
unsafe impl bytemuck::Pod for LightRaw {}

pub struct Light {
    pub raw: LightRaw,
    pub binding: Binding,
}

impl Light {
    pub fn new(
        device: &wgpu::Device,
        position: cgmath::Vector3<f32>,
        color: cgmath::Vector3<f32>,
    ) -> Self {
        let raw = LightRaw {
            position,
            _padding: 0,
            color,
        };

        let binding = BindingBuilder::new()
            .uniform_buffer(
                "light",
                &[raw],
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                false,
            )
            .build(device);

        Self { raw, binding }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        // self.binding.write_buffer_at_index(queue, 0, 0, &[self.raw]);
        self.binding
            .write_buffer_at_label(queue, "light", 0, &[self.raw]);
    }

    pub fn position(&mut self) -> &cgmath::Vector3<f32> {
        &self.raw.position
    }

    pub fn position_as_mut(&mut self) -> &mut cgmath::Vector3<f32> {
        &mut self.raw.position
    }
}

pub trait DrawLight<'a, 'b>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) where
        'b: 'a;

    fn draw_light_model(
        &mut self,
        model: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Geom,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, uniforms, light);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, uniforms, &[]);
        self.set_bind_group(1, light, &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, uniforms, light);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Geom,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), uniforms, light);
        }
    }
}
