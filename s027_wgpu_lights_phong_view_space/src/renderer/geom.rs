use anyhow::*;
use nannou::prelude::*;
use std::ops::Range;
use std::path::Path;
use wgpu::util::DeviceExt;

use super::buffer::BufferUtil;
use super::texture;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GeomVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}
unsafe impl bytemuck::Zeroable for GeomVertex {}
unsafe impl bytemuck::Pod for GeomVertex {}

impl Vertex for GeomVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<GeomVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Geom {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Geom {
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // layout: &wgpu::BindGroupLayout,
        path: P,
    ) -> Result<Self> {
        let (obj_geoms, obj_materials) = tobj::load_obj(path.as_ref(), true)?;
        let bind_group_layout = Geom::create_bind_group_layout(device);

        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;
        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            let diffuse_texture =
                texture::Texture::load(device, queue, containing_folder.join(diffuse_path))?;

            let bind_group = Geom::create_bind_group_from_texture(
                device,
                &bind_group_layout,
                &diffuse_texture.view,
                &diffuse_texture.sampler,
            );

            materials.push(Material {
                name: mat.name,
                diffuse_texture,
                bind_group,
            });
        }

        let mut meshes = Vec::new();
        for m in obj_geoms {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(GeomVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                });
            }

            let vertex_buffer = Self::create_vertex_buffer(device, &vertices);
            let index_buffer = Self::create_index_buffer(device, &m.mesh.indices);

            meshes.push(Mesh {
                name: m.name,
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            });
        }

        Ok(Self {
            meshes,
            materials,
            bind_group_layout,
        })
    }
}

impl BufferUtil for Geom {
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::BindGroupLayoutBuilder::new()
            // .label("texture_bind_group_layout")
            .sampled_texture(
                wgpu::ShaderStage::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureComponentType::Uint,
            )
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device)
    }
}

pub trait DrawGeom<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );

    fn draw_geom(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_geom_instanced(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawGeom<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, &u_instances, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_geom(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &geom.meshes {
            let material = &geom.materials[mesh.material];
            self.draw_mesh(mesh, material, uniforms, light);
        }
    }

    fn draw_geom_instanced(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &geom.meshes {
            let material = &geom.materials[mesh.material];
            self.draw_mesh_instanced(
                mesh,
                material,
                uniforms,
                light,
                u_instances,
                instances.clone(),
            );
        }
    }
}
