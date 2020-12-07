//! currently nannou's vertex and mesh doesn't have normal / tangent / bitangent
//! so use my own vertex
//!
use anyhow::*;
use image::GenericImageView;
use nannou::math::cgmath;
use nannou::prelude::*;
use rayon::prelude::*;
use std::ops::Range;
use std::path::Path;

use super::binding::{self, Binding, BindingBuilder, BindingType};
use super::material::Material;
use super::mesh::Mesh;
use super::texture::TextureSet;
use super::vertex::{Vertex, VertexDescription};

pub struct Geom {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Geom {
    pub fn load<'a, P: AsRef<Path>>(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        path: P,
    ) -> Result<Self> {
        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;
        let (obj_geoms, obj_materials) = tobj::load_obj(path.as_ref(), true)?;

        let materials = obj_materials
            .par_iter()
            .map(|mat| {
                // We can also parallelize loading the textures!
                let mut textures = [
                    containing_folder.join(&mat.diffuse_texture),
                    containing_folder.join(&mat.normal_texture),
                ]
                .par_iter()
                .map(|texture_path| {
                    let texture = wgpu::Texture::from_path((device, queue), texture_path).unwrap();
                    let view = texture.view().build();
                    let sampler = wgpu::SamplerBuilder::new()
                        .address_mode(wgpu::AddressMode::ClampToEdge)
                        .mag_filter(wgpu::FilterMode::Linear)
                        .min_filter(wgpu::FilterMode::Nearest)
                        .mipmap_filter(wgpu::FilterMode::Nearest)
                        .build(device);
                    Ok(TextureSet {
                        texture,
                        view,
                        sampler,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

                // Pop removes from the end of the list.
                let normal_texture = textures.pop().unwrap();
                let diffuse_texture = textures.pop().unwrap();

                let binding = BindingBuilder::new()
                    .texture(
                        "diffuse_texture",
                        diffuse_texture.texture,
                        diffuse_texture.view,
                        diffuse_texture.sampler,
                        wgpu::ShaderStage::FRAGMENT,
                        false,
                    )
                    .texture(
                        "normal_texture",
                        normal_texture.texture,
                        normal_texture.view,
                        normal_texture.sampler,
                        wgpu::ShaderStage::FRAGMENT,
                        false,
                    )
                    .build(device);

                Ok(Material {
                    name: mat.name.to_owned(),
                    binding,
                })
            })
            .collect::<Result<Vec<Material>>>()?;

        let meshes = obj_geoms
            .par_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .into_par_iter()
                    .map(|i| {
                        Vertex::new(
                            [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                            [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                            // We'll calculate these later
                            [0.0; 3],
                            [0.0; 3],
                        )
                    })
                    .collect::<Vec<_>>();

                let binding = BindingBuilder::new()
                    .storage_buffer_custom(
                        "vertex_buffer",
                        &vertices,
                        wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::COMPUTE,
                        false,
                        false,
                    )
                    .storage_buffer_custom(
                        "index_buffer",
                        &m.mesh.indices,
                        wgpu::BufferUsage::INDEX | wgpu::BufferUsage::STORAGE,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::COMPUTE,
                        false,
                        false,
                    )
                    .build(device);

                Ok(Mesh {
                    name: m.name.clone(),
                    vertices,
                    indices: m.mesh.indices.clone(),
                    binding,
                    material_id: m.mesh.material_id.unwrap_or(0),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Geom { meshes, materials })
    }
}

pub trait DrawGeom<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b wgpu::BindGroup);
    fn draw_mesh_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_mesh_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
    );

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_mesh_instanced_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_mesh_instanced_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );

    fn draw_geom(&mut self, geom: &'b Geom, uniforms: &'b wgpu::BindGroup);
    fn draw_geom_with_light(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_geom_with_light_and_inner_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_geom_with_light_and_other_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
    );

    fn draw_geom_instanced(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_geom_instanced_with_light(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_geom_instanced_with_light_and_inner_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_geom_instanced_with_light_and_other_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawGeom<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }

    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }

    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, &material.binding.bind_group(), &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }

    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
    }

    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh_instanced_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
    }

    // TODO: mesh.binding.buffers[0][1].....
    fn draw_mesh_instanced_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.binding.buffers[0].slice(..));
        self.set_index_buffer(mesh.binding.buffers[1].slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, material.binding.bind_group(), &[]);
        self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
    }

    fn draw_geom(&mut self, geom: &'b Geom, uniforms: &'b wgpu::BindGroup) {
        for mesh in &geom.meshes {
            self.draw_mesh(mesh, uniforms);
        }
    }

    fn draw_geom_with_light(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &geom.meshes {
            self.draw_mesh_with_light(mesh, uniforms, light);
        }
    }

    fn draw_geom_with_light_and_inner_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &geom.meshes {
            let material = &geom.materials[mesh.material_id];
            self.draw_mesh_with_light_and_material(mesh, uniforms, light, material);
        }
    }

    fn draw_geom_with_light_and_other_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
    ) {
        for mesh in &geom.meshes {
            self.draw_mesh_with_light_and_material(mesh, uniforms, light, material);
        }
    }

    fn draw_geom_instanced(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &geom.meshes {
            self.draw_mesh_instanced(mesh, uniforms, u_instances, instances.clone());
        }
    }

    fn draw_geom_instanced_with_light(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &geom.meshes {
            self.draw_mesh_instanced_with_light(
                mesh,
                uniforms,
                light,
                u_instances,
                instances.clone(),
            );
        }
    }

    fn draw_geom_instanced_with_light_and_inner_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &geom.meshes {
            let material = &geom.materials[mesh.material_id];
            self.draw_mesh_instanced_with_light_and_material(
                mesh,
                uniforms,
                light,
                material,
                u_instances,
                instances.clone(),
            );
        }
    }

    fn draw_geom_instanced_with_light_and_other_material(
        &mut self,
        geom: &'b Geom,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &geom.meshes {
            self.draw_mesh_instanced_with_light_and_material(
                mesh,
                uniforms,
                light,
                material,
                u_instances,
                instances.clone(),
            );
        }
    }
}
