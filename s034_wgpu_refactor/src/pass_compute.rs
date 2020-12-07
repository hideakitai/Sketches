use nannou::prelude::*;
use std::sync::Arc;

use crate::renderer::{
    binding::{Binding, BindingBuilder},
    vertex::Vertex,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeInfo {
    num_vertices: u32,
    num_indices: u32,
}

pub struct PassCompute {
    compute_info: ComputeInfo,
    binding: Binding,
}

impl PassCompute {
    pub fn new(
        device: &wgpu::Device,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>,
        binding_ref: &Binding,
    ) -> Self {
        let compute_info = ComputeInfo {
            num_vertices: vertices.len() as _,
            num_indices: indices.len() as _,
        };

        let binding = BindingBuilder::new()
            .storage_buffer(
                "src_vertex_buffer",
                &vertices,
                wgpu::ShaderStage::COMPUTE,
                false,
                true,
            )
            .assign_storage_buffer(
                "dst_vertex_buffer",
                Arc::clone(&binding_ref.bindings[0]),
                Arc::clone(&binding_ref.buffers[0]),
            )
            .assign_storage_buffer(
                "index_buffer",
                Arc::clone(&binding_ref.bindings[1]),
                Arc::clone(&binding_ref.buffers[1]),
            )
            .uniform_buffer(
                "compute_info",
                &[compute_info],
                wgpu::ShaderStage::COMPUTE,
                false,
            )
            .build(device);

        Self {
            compute_info,
            binding,
        }
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), ()> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Tangent and Bitangent Calc"),
        });
        {
            let shader_src = wgpu::shader_from_spirv_bytes(
                device,
                include_bytes!("../shaders/model_load.comp.spv"),
            );
            let pipeline_layout = crate::renderer::PipelineLayoutBuilder::new()
                .label("Compute Pipeline Layout")
                .bind_group_layouts(&[self.binding.bind_group_layout()])
                .build(device);
            let pipeline =
                crate::renderer::ComputePipelineBuilder::from_layout(&pipeline_layout, &shader_src)
                    .label("ModelLoader ComputePipeline")
                    .build(device);

            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, self.binding.bind_group(), &[]);
            pass.dispatch(self.compute_info.num_vertices as u32, 1, 1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);

        Ok(())
    }
}
