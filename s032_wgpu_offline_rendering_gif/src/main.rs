use nannou::prelude::*;

pub type BufferSize = std::num::NonZeroU64;

mod pass;
mod renderer;

use crate::pass::PassMain;
use bytemuck;
use futures;

struct Model {
    pass: pass::PassMain,
    last_mouse_pos: Point2<f32>,
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

fn model(app: &App) -> Model {
    // build window
    let window_id = app
        .new_window()
        .size(800, 600)
        .title("nannou")
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_moved(mouse_moved)
        .mouse_wheel(mouse_wheel)
        .resized(resized)
        .raw_view(raw_view)
        // .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();
    let sc_desc = window.swap_chain_descriptor();

    let pass = PassMain::new(device, queue, sc_desc);
    let last_mouse_pos = app.mouse.position();

    // offline render gif

    let texture_size = 256u32;
    let texture = wgpu::TextureBuilder::new()
        .extent(wgpu::Extent3d {
            width: texture_size,
            height: texture_size,
            depth: 1,
        })
        // .label(None)
        // .array_layer_count(1)
        .mip_level_count(1)
        .sample_count(1)
        .dimension(wgpu::TextureDimension::D2)
        .format(wgpu::TextureFormat::Rgba8UnormSrgb)
        .usage(wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT)
        .build(device);
    let texture_view = texture.view().build();

    // we need to store this for later
    let u32_size = std::mem::size_of::<u32>() as u32;

    // let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
    // let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: None,
    //     size: output_buffer_size,
    //     usage: wgpu::BufferUsage::COPY_DST
    //         // this tells wpgu that we want to read this buffer from the cpu
    //         | wgpu::BufferUsage::MAP_READ,
    //     mapped_at_creation: false,
    // });

    let vs_module =
        &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/offline.vert.spv"));
    let fs_module =
        &wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/offline.frag.spv"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: texture.format(),
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    // let mut frames =Vec::new();
    for i in 0..10 {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = wgpu::RenderPassBuilder::new()
                .color_attachment(&texture_view, |color| {
                    color
                        .resolve_target(None)
                        .load_op(wgpu::LoadOp::Clear(wgpu::Color {
                            r: i as f64 / 10.,
                            g: i as f64 / 20.,
                            b: 0.5,
                            a: 1.0,
                        }))
                        .store_op(true)
                })
                .begin(&mut encoder);

            render_pass.set_pipeline(&render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        // queue_gif(
        //     device,
        //     queue,
        //     encoder,
        //     &texture,
        //     &texture_view,
        //     u32_size,
        //     texture_size,
        //     texture_size,
        //     &mut frames,
        // );

        // capture_frame(
        //     device,
        //     queue,
        //     encoder,
        //     &texture,
        //     &texture_view,
        //     u32_size,
        //     texture_size,
        //     texture_size,
        // );

        let path = format!("image_{}.png", i);
        capture_frame_2(device, queue, encoder, &texture, path);
    }

    // save_gif("output.gif", &mut frames, 10, texture_size as u16).unwrap();

    Model {
        pass,
        last_mouse_pos,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let queue = window.swap_chain_queue();

    model.pass.update(queue, app.duration.since_prev_update);
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn resized(app: &App, model: &mut Model, _: Vector2) {
    let window = app.main_window();
    let device = window.swap_chain_device();
    let sc_desc = window.swap_chain_descriptor();

    model.pass.resized(device, sc_desc);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    model.pass.key_pressed(key);
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    model.pass.key_released(key);
}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {
    model.pass.mouse_moved(pos, model.last_mouse_pos);
    model.last_mouse_pos = pos;
}

fn mouse_wheel(_app: &App, model: &mut Model, dt: MouseScrollDelta, _phase: TouchPhase) {
    model.pass.mouse_wheel(&dt);
}

fn raw_view(_app: &App, model: &Model, raw_frame: RawFrame) {
    let mut encoder = raw_frame.command_encoder();
    model
        .pass
        .draw(&mut encoder, raw_frame.swap_chain_texture());
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model.pass.draw(&mut encoder, frame.texture_view());
}

#[macro_use]
extern crate lazy_static;
use futures::executor::ThreadPool;

lazy_static! {
    static ref THREAD_POOL: ThreadPool = ThreadPool::new().unwrap();
}

fn capture_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    texture_view: &wgpu::TextureView,
    bytes_per_pixel: u32,
    width: u32,
    height: u32,
) {
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: texture.size_bytes() as u64,
        usage: wgpu::BufferUsage::COPY_DST
            // this tells wpgu that we want to read this buffer from the cpu
            | wgpu::BufferUsage::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::BufferCopyView {
            buffer: &output_buffer,
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: bytes_per_pixel * width,
                rows_per_image: height,
            },
        },
        texture_view.extent(),
    );

    queue.submit(std::iter::once(encoder.finish()));

    let future = async move {
        // Note that we're not calling `.await` here.
        let buffer_slice = output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
        // let buffer_future = output_buffer.read();

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = buffer_future.await {
            let padded_buffer = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, padded_buffer).unwrap();
            buffer.save("image.png").unwrap();
        }
    };
    futures::executor::block_on(future);
    // THREAD_POOL.spawn_ok(future); // TODO: change to the thread pool

    // more smart capture with nannou!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

    // let output_buffer = texture.to_buffer(device, &mut encoder);
    // let buffer_future = output_buffer.read();

    // let future = async move {
    //     match buffer_future.await {
    //         Ok(padded_buffer) => {
    //             padded_buffer.;
    //         }
    //         Err(_) => println!("err"),
    //     }
    // };
    // futures::executor::block_on(future);
}

use std::path::Path;

fn capture_frame_2(
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

fn queue_gif(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    texture_view: &wgpu::TextureView,
    bytes_per_pixel: u32,
    width: u32,
    height: u32,
    frames: &mut Vec<Vec<u8>>,
) {
    let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = pixel_size * width;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: texture.size_bytes() as u64,
        usage: wgpu::BufferUsage::COPY_DST
            // this tells wpgu that we want to read this buffer from the cpu
            | wgpu::BufferUsage::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::BufferCopyView {
            buffer: &output_buffer,
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: padded_bytes_per_row,
                rows_per_image: height,
            },
        },
        texture_view.extent(),
    );

    queue.submit(std::iter::once(encoder.finish()));

    let future = async move {
        // Create the map request
        let buffer_slice = output_buffer.slice(..);
        let request = buffer_slice.map_async(wgpu::MapMode::Read);
        // wait for the GPU to finish
        device.poll(wgpu::Maintain::Wait);
        let result = request.await;

        match result {
            Ok(()) => {
                let padded_data = buffer_slice.get_mapped_range();
                let data = padded_data
                    .chunks(padded_bytes_per_row as _)
                    .map(|chunk| &chunk[..unpadded_bytes_per_row as _])
                    .flatten()
                    .map(|x| *x)
                    .collect::<Vec<_>>();
                drop(padded_data);
                output_buffer.unmap();
                frames.push(data);
            }
            _ => eprintln!("Something went wrong"),
        }
    };

    futures::executor::block_on(future);
}

use anyhow::*;

fn save_gif(path: &str, frames: &mut Vec<Vec<u8>>, speed: i32, size: u16) -> Result<()> {
    // use gif::{Encoder, Frame, Repeat, SetParameter};
    use gif::{Encoder, Frame, Repeat};

    let mut image = std::fs::File::create(path)?;
    let mut encoder = Encoder::new(&mut image, size, size, &[])?;
    // encoder.set(Repeat::Infinite)?;

    for mut frame in frames {
        encoder.write_frame(&Frame::from_rgba_speed(size, size, &mut frame, speed))?;
    }

    Ok(())
}
