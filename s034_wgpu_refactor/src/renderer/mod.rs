// TODO: I want to integrate custom renderer and Draw, but we cannot get some Draw's private properties
// TODO: unless make changes to nannou crate directly. So I will make my own renderer...

use nannou::prelude::*;

pub mod binding;
pub mod camera;
// pub mod draw;
pub mod geom;
pub mod instance;
pub mod light;
pub mod material;
pub mod mesh;
pub mod texture;
pub mod vertex;

pub struct PipelineLayoutBuilder<'a> {
    pipeline_desc: wgpu::PipelineLayoutDescriptor<'a>,
}

impl<'a> PipelineLayoutBuilder<'a> {
    pub fn new() -> Self {
        Self {
            pipeline_desc: wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            },
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.pipeline_desc.label = Some(label);
        self
    }

    pub fn bind_group_layouts(mut self, layouts: &'a [&wgpu::BindGroupLayout]) -> Self {
        self.pipeline_desc.bind_group_layouts = layouts;
        self
    }

    // pub fn push_constant_ranges(mut self, ranges: &'a [wgpu::PushConstantRange]) -> Self {
    //     self.pipeline_desc.push_constant_ranges = ranges;
    //     self
    // }

    pub fn build(self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&self.pipeline_desc)
    }
}

pub struct ComputePipelineBuilder<'a> {
    pipeline_desc: wgpu::ComputePipelineDescriptor<'a>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn from_layout(layout: &'a wgpu::PipelineLayout, cs_mod: &'a wgpu::ShaderModule) -> Self {
        Self {
            pipeline_desc: wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(layout),
                compute_stage: wgpu::ProgrammableStageDescriptor {
                    module: cs_mod,
                    entry_point: "main",
                },
            },
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.pipeline_desc.label = Some(label);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::ComputePipeline {
        device.create_compute_pipeline(&self.pipeline_desc)
    }
}

pub fn capture_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    path: String,
) {
    // Create the texture capturer.
    let texture_capturer = wgpu::TextureCapturer::default();

    // Take a snapshot of the texture. The capturer will do the following:
    //
    // 1. Resolve the texture to a non-multisampled texture if necessary.
    // 2. Convert the format to non-linear 8-bit sRGBA ready for image storage.
    // 3. Copy the result to a buffer ready to be mapped for reading.
    let snapshot = texture_capturer.capture(device, &mut encoder, texture);

    // Submit the commands for our drawing and texture capture to the GPU.
    queue.submit(std::iter::once(encoder.finish()));

    // Submit a function for writing our snapshot to a PNG.
    //
    // NOTE: It is essential that the commands for capturing the snapshot are `submit`ted before we
    // attempt to read the snapshot - otherwise we will read a blank texture!
    // let path = "image2.png";
    snapshot
        .read(move |result| {
            let image = result.expect("failed to map texture memory").to_owned();
            image
                .save(path)
                .expect("failed to save texture to png image");
        })
        .unwrap();
}
