use nannou::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

// TODO: nameing...
#[derive(Debug)]
#[allow(dead_code)]
pub enum BindingType {
    Uniform {
        contents: Arc<[u8]>,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
    },
    // Vertex{
    //     contents: Vec<T>,
    //     visibility: wgpu::ShaderStage,
    //     dynamic: bool,
    // },
    Storage {
        contents: Arc<[u8]>,
        usage: wgpu::BufferUsage,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
        readonly: bool,
    },
    Texture {
        texture: wgpu::Texture,
        texture_view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        visibility: wgpu::ShaderStage,
        comparison: bool,
    },
    StorageTexture {
        texture: wgpu::Texture,
        texture_view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        visibility: wgpu::ShaderStage,
        readonly: bool,
    },
    SharedUniformBuffer {
        binding: Arc<BindingType>,
        buffer: Arc<wgpu::Buffer>,
    },
    SharedStorageBuffer {
        binding: Arc<BindingType>,
        buffer: Arc<wgpu::Buffer>,
    },
    SharedTexture {
        binding: Arc<BindingType>,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
        sampler: Arc<wgpu::Sampler>,
    },
    SharedStorageTexture {
        binding: Arc<BindingType>,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
        sampler: Arc<wgpu::Sampler>,
    },
}

// TODO: naming....
pub struct Binding {
    pub label_index: HashMap<String, usize>,
    pub bindings: Vec<Arc<BindingType>>,
    pub buffers: Vec<Arc<wgpu::Buffer>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Binding {
    pub fn new(
        device: &wgpu::Device,
        labels: Vec<String>,
        bindings: Vec<Arc<BindingType>>,
    ) -> Binding {
        assert_eq!(
            labels.len(),
            bindings.len(),
            "The number of labels and bindings must be same"
        );
        let label_index = labels
            .into_iter()
            .enumerate()
            .map(|(i, label)| (label, i))
            .collect::<HashMap<String, usize>>();
        let buffers = Self::create_buffers(device, &bindings);
        let bind_group_layout = Self::create_bind_group_layout(device, &bindings);
        let bind_group = Self::create_bind_group(device, &bindings, &buffers, &bind_group_layout);
        Self {
            label_index,
            bindings,
            buffers,
            bind_group_layout,
            bind_group,
        }
    }

    // TODO: add label
    fn create_buffers(
        device: &wgpu::Device,
        bindings: &Vec<Arc<BindingType>>,
    ) -> Vec<Arc<wgpu::Buffer>> {
        let mut buffers = vec![];
        for binding in bindings.iter() {
            match binding.as_ref() {
                BindingType::Uniform { contents, .. } => {
                    buffers.push(Arc::new(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents,
                            usage: wgpu::BufferUsage::UNIFORM
                                | wgpu::BufferUsage::COPY_SRC
                                | wgpu::BufferUsage::COPY_DST,
                        },
                    )));
                }
                BindingType::Storage {
                    contents, usage, ..
                } => {
                    buffers.push(Arc::new(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents,
                            usage: *usage,
                        },
                    )));
                }
                BindingType::SharedUniformBuffer { buffer, .. } => {
                    buffers.push(Arc::clone(&buffer));
                }
                BindingType::SharedStorageBuffer { buffer, .. } => {
                    buffers.push(Arc::clone(&buffer));
                }
                _ => {
                    buffers.push(Arc::new(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Dummy"),
                            contents: &[],
                            usage: wgpu::BufferUsage::STORAGE,
                        },
                    )));
                }
            }
        }
        buffers
    }

    fn create_bind_group_layout(
        device: &wgpu::Device,
        bindings: &Vec<Arc<BindingType>>,
    ) -> wgpu::BindGroupLayout {
        let mut builder = wgpu::BindGroupLayoutBuilder::new();
        for binding in bindings.iter() {
            match binding.as_ref() {
                BindingType::Uniform {
                    visibility,
                    dynamic,
                    ..
                } => {
                    builder = builder.uniform_buffer(*visibility, *dynamic);
                }
                BindingType::Storage {
                    visibility,
                    dynamic,
                    readonly,
                    ..
                } => {
                    builder = builder.storage_buffer(*visibility, *dynamic, *readonly);
                }
                BindingType::Texture {
                    texture,
                    visibility,
                    comparison,
                    ..
                } => {
                    if *comparison {
                        builder = builder
                            .sampled_texture_from(*visibility, texture)
                            .comparison_sampler(*visibility);
                    } else {
                        builder = builder
                            .sampled_texture_from(*visibility, texture)
                            .sampler(*visibility);
                    }
                }
                BindingType::StorageTexture {
                    texture,
                    visibility,
                    readonly,
                    ..
                } => {
                    builder = builder.storage_texture_from(*visibility, texture, *readonly);
                }
                BindingType::SharedUniformBuffer { binding, .. } => {
                    if let BindingType::Uniform {
                        visibility,
                        dynamic,
                        ..
                    } = binding.as_ref()
                    {
                        builder = builder.uniform_buffer(*visibility, *dynamic);
                    } else {
                        panic!("assigned buffer type is not matched");
                    };
                }
                BindingType::SharedStorageBuffer { binding, .. } => {
                    if let BindingType::Storage {
                        visibility,
                        dynamic,
                        readonly,
                        ..
                    } = binding.as_ref()
                    {
                        builder = builder.storage_buffer(*visibility, *dynamic, *readonly);
                    } else {
                        panic!("assigned buffer type is not matched");
                    };
                }
                BindingType::SharedTexture { binding, .. } => {
                    if let BindingType::Texture {
                        texture,
                        visibility,
                        comparison,
                        ..
                    } = binding.as_ref()
                    {
                        if *comparison {
                            builder = builder
                                .sampled_texture_from(*visibility, texture)
                                .comparison_sampler(*visibility);
                        } else {
                            builder = builder
                                .sampled_texture_from(*visibility, texture)
                                .sampler(*visibility);
                        }
                    } else {
                        panic!("assigned buffer type is not matched");
                    };
                }
                BindingType::SharedStorageTexture { binding, .. } => {
                    if let BindingType::StorageTexture {
                        texture,
                        visibility,
                        readonly,
                        ..
                    } = binding.as_ref()
                    {
                        builder = builder.storage_texture_from(*visibility, texture, *readonly);
                    } else {
                        panic!("assigned buffer type is not matched");
                    };
                }
            }
        }
        builder.build(device)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        bindings: &Vec<Arc<BindingType>>,
        buffers: &Vec<Arc<wgpu::Buffer>>,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let mut builder = wgpu::BindGroupBuilder::new();
        for (binding, buffer) in bindings.iter().zip(buffers.iter()) {
            match binding.as_ref() {
                BindingType::Uniform { .. } => {
                    builder = builder.binding(wgpu::BindingResource::Buffer(buffer.slice(..)));
                }
                BindingType::Storage { .. } => {
                    builder = builder.binding(wgpu::BindingResource::Buffer(buffer.slice(..)));
                }
                BindingType::Texture {
                    texture_view,
                    sampler,
                    ..
                } => {
                    builder = builder.texture_view(texture_view).sampler(sampler);
                }
                BindingType::StorageTexture {
                    texture_view,
                    sampler,
                    ..
                } => {
                    builder = builder.texture_view(texture_view).sampler(sampler);
                }
                BindingType::SharedUniformBuffer { .. } => {
                    builder = builder.binding(wgpu::BindingResource::Buffer(buffer.slice(..)));
                }
                BindingType::SharedStorageBuffer { .. } => {
                    builder = builder.binding(wgpu::BindingResource::Buffer(buffer.slice(..)));
                }
                BindingType::SharedTexture {
                    texture_view,
                    sampler,
                    ..
                } => {
                    builder = builder
                        .texture_view(texture_view.as_ref())
                        .sampler(sampler.as_ref());
                }
                BindingType::SharedStorageTexture {
                    texture_view,
                    sampler,
                    ..
                } => {
                    builder = builder
                        .texture_view(texture_view.as_ref())
                        .sampler(sampler.as_ref());
                }
            }
        }
        builder.build(device, bind_group_layout)
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn write_buffer_at_index<T: bytemuck::Pod>(
        &mut self,
        queue: &wgpu::Queue,
        index: usize,
        offset: wgpu::BufferAddress,
        data: &[T],
    ) {
        match &self.bindings[index].as_ref() {
            BindingType::Uniform { .. } => {
                queue.write_buffer(&self.buffers[index], offset, bytemuck::cast_slice(&data));
            }
            BindingType::Storage { .. } => {
                queue.write_buffer(&self.buffers[index], offset, bytemuck::cast_slice(&data));
            }
            BindingType::SharedUniformBuffer { .. } => {
                queue.write_buffer(&self.buffers[index], offset, bytemuck::cast_slice(&data));
            }
            BindingType::SharedStorageBuffer { .. } => {
                queue.write_buffer(&self.buffers[index], offset, bytemuck::cast_slice(&data));
            }
            _ => {
                panic!("There is no buffer to write to in index {}", index);
            }
        }
    }

    pub fn write_texture_at_index(&self, queue: &wgpu::Queue, index: usize, data: &[u8]) {
        match &self.bindings[index].as_ref() {
            BindingType::Texture { texture, .. } => {
                self.write_texture(queue, texture, data);
            }
            BindingType::StorageTexture { texture, .. } => {
                self.write_texture(queue, texture, data);
            }
            BindingType::SharedTexture { texture, .. } => {
                self.write_texture(queue, texture, data);
            }
            BindingType::SharedStorageTexture { texture, .. } => {
                self.write_texture(queue, texture, data);
            }
            _ => {
                panic!("There is no texture to write to in index {}", index);
            }
        }
    }

    pub fn write_buffer_at_label<T: bytemuck::Pod>(
        &mut self,
        queue: &wgpu::Queue,
        label: &str,
        offset: wgpu::BufferAddress,
        data: &[T],
    ) {
        if let Some(index) = self.label_index.get(label) {
            self.write_buffer_at_index(queue, *index, offset, data);
        } else {
            panic!("There is no label {}", label);
        }
    }

    pub fn write_texture_at_label(&self, queue: &wgpu::Queue, label: &str, data: &[u8]) {
        if let Some(index) = self.label_index.get(label) {
            self.write_texture_at_index(queue, *index, data);
        } else {
            panic!("There is no label {}", label);
        }
    }

    fn write_texture(&self, queue: &wgpu::Queue, texture: &wgpu::Texture, data: &[u8]) {
        let mip_level = texture.mip_level_count();
        let rows_per_image = texture.size()[1];
        let bytes_per_row = texture.size_bytes() as u32 / rows_per_image;
        let size = texture.extent();
        queue.write_texture(
            wgpu::TextureCopyView {
                texture,
                mip_level,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row,
                rows_per_image,
            },
            size,
        );
    }
}

pub struct BindingBuilder {
    pub labels: Vec<String>,
    pub bindings: Vec<Arc<BindingType>>,
}

#[allow(dead_code)]
impl BindingBuilder {
    pub fn new() -> Self {
        Self {
            labels: vec![],
            bindings: vec![],
        }
    }

    // TODO: &[T] -> &T ?? currently I can't cast &[T] to &[u8] correctly... need bytemuck to be generic
    // pub fn uniform_buffer<T>(
    pub fn uniform_buffer<T: bytemuck::Pod>(
        mut self,
        label: &str,
        contents: &[T],
        // contents: &T,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::Uniform {
            contents: bytemuck::cast_slice(&contents).into(),
            // contents: unsafe { wgpu::bytes::from_slice(&[contents]) }.into(),
            visibility,
            dynamic,
        }));
        self
    }

    // TODO: &[T] -> &T ?? currently I can't cast &[T] to &[u8] correctly... need bytemuck to be generic
    // pub fn storage_buffer<T>(
    pub fn storage_buffer<T: bytemuck::Pod>(
        mut self,
        label: &str,
        contents: &[T],
        // contents: &T,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
        readonly: bool,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::Storage {
            contents: bytemuck::cast_slice(&contents).into(),
            // contents: unsafe { wgpu::bytes::from_slice(&[contents]) }.into(),
            usage: wgpu::BufferUsage::STORAGE,
            visibility,
            dynamic,
            readonly,
        }));
        self
    }

    pub fn storage_buffer_custom<T: bytemuck::Pod>(
        mut self,
        label: &str,
        contents: &[T],
        usage: wgpu::BufferUsage,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
        readonly: bool,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::Storage {
            contents: bytemuck::cast_slice(&contents).into(),
            usage,
            visibility,
            dynamic,
            readonly,
        }));
        self
    }

    pub fn texture(
        mut self,
        label: &str,
        texture: wgpu::Texture,
        texture_view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        visibility: wgpu::ShaderStage,
        comparison: bool,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::Texture {
            texture,
            texture_view,
            sampler,
            visibility,
            comparison,
        }));
        self
    }

    pub fn storage_texture(
        mut self,
        label: &str,
        texture: wgpu::Texture,
        texture_view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        visibility: wgpu::ShaderStage,
        readonly: bool,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::StorageTexture {
            texture,
            texture_view,
            sampler,
            visibility,
            readonly,
        }));
        self
    }

    pub fn assign_uniform_buffer(
        mut self,
        label: &str,
        binding: Arc<BindingType>,
        buffer: Arc<wgpu::Buffer>,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings
            .push(Arc::new(BindingType::SharedUniformBuffer {
                binding,
                buffer,
            }));
        self
    }

    pub fn assign_storage_buffer(
        mut self,
        label: &str,
        binding: Arc<BindingType>,
        buffer: Arc<wgpu::Buffer>,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings
            .push(Arc::new(BindingType::SharedStorageBuffer {
                binding,
                buffer,
            }));
        self
    }

    pub fn assign_texture(
        mut self,
        label: &str,
        binding: Arc<BindingType>,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
        sampler: Arc<wgpu::Sampler>,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings.push(Arc::new(BindingType::SharedTexture {
            binding,
            texture,
            texture_view,
            sampler,
        }));
        self
    }

    pub fn assign_storage_texture(
        mut self,
        label: &str,
        binding: Arc<BindingType>,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
        sampler: Arc<wgpu::Sampler>,
    ) -> Self {
        self.labels.push(label.to_string());
        self.bindings
            .push(Arc::new(BindingType::SharedStorageTexture {
                binding,
                texture,
                texture_view,
                sampler,
            }));
        self
    }

    pub fn build(self, device: &wgpu::Device) -> Binding {
        Binding::new(device, self.labels, self.bindings)
    }
}
