use super::binding::{Binding, BindingBuilder};
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
    pub binding: Binding,
}

impl Instances {
    // pub fn new<F: FnMut(u32) -> Instance>(device: &wgpu::Device, num: u32, f: F) -> Self {
    //     let instances = (0..num)
    //         .flat_map(|z| (0..num).map(f))
    //         .collect::<Vec<Instance>>();
    //     let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    //     let binding = BindingBuilder::new()
    //         .storage_buffer(&instance_data, wgpu::ShaderStage::VERTEX, false, true)
    //         .build(device);

    //     Self {
    //         instances: instances.to_vec(),
    //         binding,
    //     }
    // }

    pub fn from_vec(device: &wgpu::Device, instances: &[Instance]) -> Self {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let binding = BindingBuilder::new()
            .storage_buffer(
                "instance_buffer",
                &instance_data,
                wgpu::ShaderStage::VERTEX,
                false,
                true,
            )
            .build(device);

        Self {
            instances: instances.to_vec(),
            binding,
        }
    }
}
