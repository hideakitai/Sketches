use anyhow::*;
use nannou::math::cgmath;
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
    // position: [f32; 3],
    // tex_coords: [f32; 2],
    // normal: [f32; 3],
    position: cgmath::Vector3<f32>,
    tex_coords: cgmath::Vector2<f32>,
    normal: cgmath::Vector3<f32>,
    tangent: cgmath::Vector3<f32>,
    bitangent: cgmath::Vector3<f32>,
}
unsafe impl bytemuck::Zeroable for GeomVertex {}
unsafe impl bytemuck::Pod for GeomVertex {}

impl Vertex for GeomVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<GeomVertex>() as wgpu::BufferAddress,
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

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
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
                texture::Texture::load(device, queue, containing_folder.join(diffuse_path), false)?;

            let normal_path = mat.normal_texture;
            let normal_texture =
                texture::Texture::load(device, queue, containing_folder.join(normal_path), true)?;

            let bind_group = Geom::create_bind_group_from_textures(
                device,
                &bind_group_layout,
                &[&diffuse_texture.view, &normal_texture.view],
                &[&diffuse_texture.sampler, &normal_texture.sampler],
            );

            materials.push(Material {
                name: mat.name,
                diffuse_texture,
                normal_texture,
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
                    ]
                    .into(),
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ]
                    .into(),
                    tangent: [0.; 3].into(),
                    bitangent: [0.; 3].into(),
                });
            }

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in m.mesh.indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0 = v0.position;
                let pos1 = v1.position;
                let pos2 = v2.position;

                let uv0 = v0.tex_coords;
                let uv1 = v1.tex_coords;
                let uv2 = v2.tex_coords;

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent = tangent;
                vertices[c[1] as usize].tangent = tangent;
                vertices[c[2] as usize].tangent = tangent;

                vertices[c[0] as usize].bitangent = bitangent;
                vertices[c[1] as usize].bitangent = bitangent;
                vertices[c[2] as usize].bitangent = bitangent;
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
            // diffuse texture
            .sampled_texture(
                wgpu::ShaderStage::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureComponentType::Uint,
            )
            .sampler(wgpu::ShaderStage::FRAGMENT)
            // normal map
            .sampled_texture(
                wgpu::ShaderStage::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureComponentType::Float,
            )
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device)
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
    fn draw_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_mesh_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_mesh_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_mesh_instanced_with_light(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_mesh_instanced_with_light_and_material(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        material: &'b Material,
        u_instances: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &u_instances, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
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
            let material = &geom.materials[mesh.material];
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
            let material = &geom.materials[mesh.material];
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
