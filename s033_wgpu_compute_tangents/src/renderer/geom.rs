use anyhow::*;
use nannou::math::cgmath;
use nannou::prelude::*;
use rayon::prelude::*;
use std::ops::Range;
use std::path::Path;
use wgpu::util::DeviceExt;

use super::buffer::{self, BufferUtil};
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeInfo {
    num_vertices: u32,
    num_indices: u32,
}

struct BitangentComputeBinding {
    src_vertex_buffer: wgpu::Buffer,
    dst_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    info_buffer: wgpu::Buffer,
    compute_info: ComputeInfo,
}

// impl BitangentComputeBinding {
//     fn new() {}
// }

impl BufferUtil for BitangentComputeBinding {
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::BindGroupLayoutBuilder::new()
            // .label(None)
            .storage_buffer(wgpu::ShaderStage::COMPUTE, false, true)
            .storage_buffer(wgpu::ShaderStage::COMPUTE, false, false)
            .storage_buffer(wgpu::ShaderStage::COMPUTE, false, true)
            .uniform_buffer(wgpu::ShaderStage::COMPUTE, false)
            .build(device)
    }
}

// impl pipeline::Bindable for BitangentComputeBinding {
//     fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
//         vec![
//             // Src Vertices
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStage::COMPUTE,
//                 ty: wgpu::BindingType::StorageBuffer {
//                     dynamic: false,
//                     min_binding_size: None,
//                     // We use these vertices to compute the tangent and bitangent
//                     readonly: true,
//                 },
//                 count: None,
//             },
//             // Dst Vertices
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStage::COMPUTE,
//                 ty: wgpu::BindingType::StorageBuffer {
//                     dynamic: false,
//                     min_binding_size: None,
//                     // We'll store the computed tangent and bitangent here
//                     readonly: false,
//                 },
//                 count: None,
//             },
//             // Indices
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStage::COMPUTE,
//                 ty: wgpu::BindingType::StorageBuffer {
//                     dynamic: false,
//                     min_binding_size: None,
//                     // We WILL NOT change the indices in the compute shader
//                     readonly: true,
//                 },
//                 count: None,
//             },
//             // ComputeInfo
//             wgpu::BindGroupLayoutEntry {
//                 binding: 3,
//                 visibility: wgpu::ShaderStage::COMPUTE,
//                 ty: wgpu::BindingType::UniformBuffer {
//                     dynamic: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             },
//         ]
//     }

//     fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
//         vec![
//             // Src Vertices
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Buffer(self.src_vertex_buffer.slice(..)),
//             },
//             // Dst Vertices
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::Buffer(self.dst_vertex_buffer.slice(..)),
//             },
//             // Indices
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: wgpu::BindingResource::Buffer(self.index_buffer.slice(..)),
//             },
//             // ComputeInfo
//             wgpu::BindGroupEntry {
//                 binding: 3,
//                 resource: wgpu::BindingResource::Buffer(self.info_buffer.slice(..)),
//             },
//         ]
//     }
// }

pub struct Geom {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct GeomLoader {
    // binder: pipeline::Binder<BitangentComputeBinding>,
// binder: BitangentComputeBinding,
// pipeline: wgpu::ComputePipeline,
}

impl GeomLoader {
    // pub fn new(device: &wgpu::Device) -> Self {
    //     // let binder = pipeline::Binder::new(device, Some("ModelLoader Binder"));
    //     let binder = BitangentComputeBinding::new(device, Some("ModelLoader Binder"));
    //     let shader_src = wgpu::include_spirv!("model_load.comp.spv");
    //     let pipeline = crate::renderer::create_compute_pipeline(
    //         device,
    //         &[&binder.layout],
    //         shader_src,
    //         Some("ModelLoader ComputePipeline"),
    //     );
    //     Self { binder, pipeline }
    // }

    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // layout: &wgpu::BindGroupLayout,
        path: P,
    ) -> Result<Geom> {
        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;
        let bind_group_layout = Geom::create_bind_group_layout(device);
        let (obj_geoms, obj_materials) = tobj::load_obj(path.as_ref(), true)?;

        let materials = obj_materials
            .par_iter()
            .map(|mat| {
                // We can also parallelize loading the textures!
                let mut textures = [
                    (containing_folder.join(&mat.diffuse_texture), false),
                    (containing_folder.join(&mat.normal_texture), true),
                ]
                .par_iter()
                .map(|(texture_path, is_normal_map)| {
                    texture::Texture::load(device, queue, texture_path, *is_normal_map)
                })
                .collect::<Result<Vec<_>>>()?;

                // Pop removes from the end of the list.
                let normal_texture = textures.pop().unwrap();
                let diffuse_texture = textures.pop().unwrap();

                let bind_group = buffer::create_bind_group_from_textures(
                    device,
                    &bind_group_layout,
                    &[&diffuse_texture.view, &normal_texture.view],
                    &[&diffuse_texture.sampler, &normal_texture.sampler],
                );

                Ok(Material {
                    name: mat.name.to_owned(),
                    diffuse_texture,
                    normal_texture,
                    bind_group,
                })
            })
            .collect::<Result<Vec<Material>>>()?;

        // let mut meshes = Vec::new();
        // for m in obj_geoms {
        //     let mut vertices = Vec::new();
        //     for i in 0..m.mesh.positions.len() / 3 {
        //         vertices.push(GeomVertex {
        //             position: [
        //                 m.mesh.positions[i * 3],
        //                 m.mesh.positions[i * 3 + 1],
        //                 m.mesh.positions[i * 3 + 2],
        //             ]
        //             .into(),
        //             tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
        //             normal: [
        //                 m.mesh.normals[i * 3],
        //                 m.mesh.normals[i * 3 + 1],
        //                 m.mesh.normals[i * 3 + 2],
        //             ]
        //             .into(),
        //             tangent: [0.; 3].into(),
        //             bitangent: [0.; 3].into(),
        //         });
        //     }

        //     // Calculate tangents and bitangets. We're going to
        //     // use the triangles, so we need to loop through the
        //     // indices in chunks of 3
        //     for c in m.mesh.indices.chunks(3) {
        //         let v0 = vertices[c[0] as usize];
        //         let v1 = vertices[c[1] as usize];
        //         let v2 = vertices[c[2] as usize];

        //         let pos0 = v0.position;
        //         let pos1 = v1.position;
        //         let pos2 = v2.position;

        //         let uv0 = v0.tex_coords;
        //         let uv1 = v1.tex_coords;
        //         let uv2 = v2.tex_coords;

        //         // Calculate the edges of the triangle
        //         let delta_pos1 = pos1 - pos0;
        //         let delta_pos2 = pos2 - pos0;

        //         // This will give us a direction to calculate the
        //         // tangent and bitangent
        //         let delta_uv1 = uv1 - uv0;
        //         let delta_uv2 = uv2 - uv0;

        //         // Solving the following system of equations will
        //         // give us the tangent and bitangent.
        //         //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //         //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        //         // Luckily, the place I found this equation provided
        //         // the solution!
        //         let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        //         let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        //         let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

        //         // We'll use the same tangent/bitangent for each vertex in the triangle
        //         vertices[c[0] as usize].tangent = tangent;
        //         vertices[c[1] as usize].tangent = tangent;
        //         vertices[c[2] as usize].tangent = tangent;

        //         vertices[c[0] as usize].bitangent = bitangent;
        //         vertices[c[1] as usize].bitangent = bitangent;
        //         vertices[c[2] as usize].bitangent = bitangent;
        //     }

        //     let vertex_buffer = Self::create_vertex_buffer(device, &vertices);
        //     let index_buffer = Self::create_index_buffer(device, &m.mesh.indices);

        //     meshes.push(Mesh {
        //         name: m.name,
        //         vertex_buffer,
        //         index_buffer,
        //         num_elements: m.mesh.indices.len() as u32,
        //         material: m.mesh.material_id.unwrap_or(0),
        //     });
        // }
        let meshes = obj_geoms
            .par_iter()
            .map(|m| {
                let mut vertices = (0..m.mesh.positions.len() / 3)
                    .into_par_iter()
                    .map(|i| {
                        GeomVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ]
                            .into(),
                            tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]]
                                .into(),
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ]
                            .into(),
                            // We'll calculate these later
                            tangent: [0.0; 3].into(),
                            bitangent: [0.0; 3].into(),
                        }
                    })
                    .collect::<Vec<_>>();

                let indices = &m.mesh.indices;

                // // Calculate tangents and bitangets. We're going to
                // // use the triangles, so we need to loop through the
                // // indices in chunks of 3
                // for c in indices.chunks(3) {
                //     let v0 = vertices[c[0] as usize];
                //     let v1 = vertices[c[1] as usize];
                //     let v2 = vertices[c[2] as usize];

                //     let pos0 = v0.position;
                //     let pos1 = v1.position;
                //     let pos2 = v2.position;

                //     let uv0 = v0.tex_coords;
                //     let uv1 = v1.tex_coords;
                //     let uv2 = v2.tex_coords;

                //     // Calculate the edges of the triangle
                //     let delta_pos1 = pos1 - pos0;
                //     let delta_pos2 = pos2 - pos0;

                //     // This will give us a direction to calculate the
                //     // tangent and bitangent
                //     let delta_uv1 = uv1 - uv0;
                //     let delta_uv2 = uv2 - uv0;

                //     // Solving the following system of equations will
                //     // give us the tangent and bitangent.
                //     //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                //     // Luckily, the place I found this equation provided
                //     // the solution!
                //     let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                //     let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                //     let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                //     // We'll use the same tangent/bitangent for each vertex in the triangle
                //     vertices[c[0] as usize].tangent = tangent;
                //     vertices[c[1] as usize].tangent = tangent;
                //     vertices[c[2] as usize].tangent = tangent;

                //     vertices[c[0] as usize].bitangent = bitangent;
                //     vertices[c[1] as usize].bitangent = bitangent;
                //     vertices[c[2] as usize].bitangent = bitangent;
                // }

                // // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                // //     label: Some(&format!("{:?} Vertex Buffer", m.name)),
                // //     contents: bytemuck::cast_slice(&vertices),
                // //     usage: wgpu::BufferUsage::VERTEX,
                // // });
                // // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                // //     label: Some(&format!("{:?} Index Buffer", m.name)),
                // //     contents: bytemuck::cast_slice(&m.mesh.indices),
                // //     usage: wgpu::BufferUsage::INDEX,
                // // });
                // let vertex_buffer = Self::create_vertex_buffer(device, &vertices);
                // let index_buffer = Self::create_index_buffer(device, &m.mesh.indices);

                // Ok(Mesh {
                //     name: m.name.clone(),
                //     vertex_buffer,
                //     index_buffer,
                //     num_elements: m.mesh.indices.len() as u32,
                //     material: m.mesh.material_id.unwrap_or(0),
                // })

                // let src_vertex_buffer =
                //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //         label: Some(&format!("{:?} Vertex Buffer", m.name)),
                //         contents: bytemuck::cast_slice(&vertices),
                //         // UPDATED!
                //         usage: wgpu::BufferUsage::STORAGE,
                //     });
                println!("vertices: {}", vertices.len());
                let src_vertex_buffer = buffer::create_storage_buffer(device, vertices.as_slice());
                // let dst_vertex_buffer =
                //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //         label: Some(&format!("{:?} Vertex Buffer", m.name)),
                //         contents: bytemuck::cast_slice(&vertices),
                //         // UPDATED!
                //         usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE,
                //     });
                println!("src_vertex_buffer: {:?}", src_vertex_buffer);
                let dst_vertex_buffer = buffer::create_buffer(
                    device,
                    vertices.as_slice(),
                    wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE,
                );
                println!("dst_vertex_buffer: {:?}", dst_vertex_buffer);
                // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //     label: Some(&format!("{:?} Index Buffer", m.name)),
                //     contents: bytemuck::cast_slice(&m.mesh.indices),
                //     // UPDATED!
                //     usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::STORAGE,
                // });
                let index_buffer = buffer::create_buffer(
                    device,
                    &indices,
                    wgpu::BufferUsage::INDEX | wgpu::BufferUsage::STORAGE,
                );
                let compute_info = ComputeInfo {
                    num_vertices: vertices.len() as _,
                    num_indices: indices.len() as _,
                };
                // let info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //     label: Some(&format!("{:?} Compute Info Buffer", m.name)),
                //     contents: bytemuck::cast_slice(&[compute_info]),
                //     usage: wgpu::BufferUsage::UNIFORM,
                // });
                let info_buffer = buffer::create_uniform_buffer(device, &[compute_info]);

                // NEW!
                // We'll need the mesh for the tangent/bitangent calculation
                let binding = BitangentComputeBinding {
                    dst_vertex_buffer,
                    src_vertex_buffer,
                    index_buffer,
                    info_buffer,
                    compute_info,
                };

                // Calculate the tangents and bitangents
                let calc_bind_group_layout =
                    BitangentComputeBinding::create_bind_group_layout(device);
                // let calc_bind_group =
                //     buffer::create_bind_group(device, calc_bind_group_layout, &binding);
                //         let buffers = create_bind_group_from_buffer
                //         let buffers =         vec![
                //             buffer::create_
                //     // Src Vertices
                //     wgpu::BindGroupEntry {
                //         binding: 0,
                //         resource: wgpu::BindingResource::Buffer(self.src_vertex_buffer.slice(..)),
                //     },
                //     // Dst Vertices
                //     wgpu::BindGroupEntry {
                //         binding: 1,
                //         resource: wgpu::BindingResource::Buffer(self.dst_vertex_buffer.slice(..)),
                //     },
                //     // Indices
                //     wgpu::BindGroupEntry {
                //         binding: 2,
                //         resource: wgpu::BindingResource::Buffer(self.index_buffer.slice(..)),
                //     },
                //     // ComputeInfo
                //     wgpu::BindGroupEntry {
                //         binding: 3,
                //         resource: wgpu::BindingResource::Buffer(self.info_buffer.slice(..)),
                //     },
                // ];
                let calc_bind_group = buffer::create_bind_group_from_buffers(
                    device,
                    &calc_bind_group_layout,
                    &[
                        &binding.src_vertex_buffer,
                        &binding.dst_vertex_buffer,
                        &binding.index_buffer,
                        &binding.info_buffer,
                    ],
                );
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Tangent and Bitangent Calc"),
                });
                {
                    let shader_src = wgpu::shader_from_spirv_bytes(
                        device,
                        include_bytes!("../../shaders/model_load.comp.spv"),
                    );
                    // let shader_src = wgpu::include_spirv!("model_load.comp.spv");
                    let pipeline = crate::renderer::create_compute_pipeline(
                        device,
                        &[&calc_bind_group_layout],
                        &shader_src,
                        Some("ModelLoader ComputePipeline"),
                    );

                    let mut pass = encoder.begin_compute_pass();
                    // pass.set_pipeline(&self.pipeline);
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &calc_bind_group, &[]);
                    pass.dispatch(binding.compute_info.num_vertices as u32, 1, 1);
                }
                queue.submit(std::iter::once(encoder.finish()));
                device.poll(wgpu::Maintain::Wait);

                Ok(Mesh {
                    name: m.name.clone(),
                    vertex_buffer: binding.dst_vertex_buffer,
                    index_buffer: binding.index_buffer,
                    num_elements: binding.compute_info.num_indices,
                    material: m.mesh.material_id.unwrap_or(0),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Geom {
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
