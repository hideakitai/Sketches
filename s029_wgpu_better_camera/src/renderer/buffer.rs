use super::geom::GeomVertex;
use super::instance::InstanceRaw;
use nannou::prelude::*;

pub trait BufferUtil {
    // TODO: make BufferBuilder
    fn create_uniform_buffer<T: bytemuck::Pod>(
        device: &wgpu::Device,
        contents: &T,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[*contents]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        })
    }

    fn create_uniform_buffer_copiable<T: bytemuck::Pod>(
        device: &wgpu::Device,
        contents: &T,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Copiable Uniform Buffer"),
            contents: bytemuck::cast_slice(&[*contents]),
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        })
    }

    fn create_vertex_buffer(device: &wgpu::Device, vertices: &[GeomVertex]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        })
    }

    fn create_index_buffer(device: &wgpu::Device, indices: &[u32]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        })
    }

    fn create_instance_buffer(
        device: &wgpu::Device,
        instance_data: &[InstanceRaw],
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsage::STORAGE,
        })
    }

    fn create_buffer<T: bytemuck::Pod>(
        device: &wgpu::Device,
        contents: &T,
        usage: wgpu::BufferUsage,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[*contents]),
            usage,
        })
    }

    fn create_buffer_from_descriptor(
        device: &wgpu::Device,
        desc: &wgpu::util::BufferInitDescriptor,
    ) -> wgpu::Buffer {
        device.create_buffer_init(desc)
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;

    fn create_bind_group_from_buffers(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[&wgpu::Buffer],
    ) -> wgpu::BindGroup {
        let mut builder = wgpu::BindGroupBuilder::new();
        for buffer in buffers.iter() {
            builder = builder.binding(wgpu::BindingResource::Buffer(buffer.slice(..)));
        }
        builder.build(device, layout)
    }

    fn create_bind_group_from_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        wgpu::BindGroupBuilder::new()
            .texture_view(texture_view)
            .sampler(sampler)
            .build(device, layout)
    }

    fn create_bind_group_from_textures(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture_views: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
    ) -> wgpu::BindGroup {
        assert_eq!(
            texture_views.len(),
            samplers.len(),
            "Number of texture / sampler must be same"
        );
        let mut builder = wgpu::BindGroupBuilder::new();
        for (view, sampler) in texture_views.iter().zip(samplers.iter()) {
            builder = builder.texture_view(view).sampler(sampler)
        }
        builder.build(device, layout)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: &[wgpu::BindingResource],
    ) -> wgpu::BindGroup {
        let mut builder = wgpu::BindGroupBuilder::new();
        for resource in resources.to_vec() {
            builder = builder.binding(resource);
        }
        builder.build(device, layout)
    }
}
