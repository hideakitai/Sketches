use super::buffer::BufferUtil;
use nannou::math::cgmath;
use nannou::prelude::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {
    model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

#[derive(Copy, Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let translation = cgmath::Matrix4::from_translation(self.position);
        let rotation = cgmath::Matrix4::from(self.rotation);
        InstanceRaw {
            model: translation * rotation,
        }
    }
}

pub struct Instances {
    pub instances: Vec<Instance>,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Instances {
    // pub fn new<F: FnMut(u32) -> Instance>(device: &wgpu::Device, num: u32, f: F) -> Self {
    //     let instances = (0..num)
    //         .flat_map(|z| (0..num).map(f))
    //         .collect::<Vec<Instance>>();

    //     let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    //     let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Instance Buffer"),
    //         contents: bytemuck::cast_slice(&instance_data),
    //         usage: wgpu::BufferUsage::STORAGE,
    //     });
    // }

    pub fn from_vec(device: &wgpu::Device, instances: &[Instance]) -> Self {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&instance_data),
        //     usage: wgpu::BufferUsage::STORAGE,
        // });
        let buffer = Self::create_instance_buffer(device, &instance_data);
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group =
            Self::create_bind_group_from_buffers(device, &bind_group_layout, &[&buffer]);

        Self {
            // instances: instances.iter().collect(),
            instances: instances.to_vec(),
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}

impl BufferUtil for Instances {
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::BindGroupLayoutBuilder::new()
            // .label(Some("uniform_bind_group_layout")),
            .storage_buffer(wgpu::ShaderStage::VERTEX, false, true)
            .build(device)
    }
}
