use nannou::math::cgmath::{self, Matrix3, Matrix4, Point3, Rad, Vector3};
use nannou::prelude::*;
use std::cell::RefCell;

mod camera;
mod data;
mod graphics;

use camera::Camera;
use graphics::Graphics;

struct Model {
    camera_is_active: bool,
    is_capturing: bool,
    capture_begin_frame: u64,
    graphics: RefCell<graphics::Graphics>,
    camera: Camera,
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

fn model(app: &App) -> Model {
    // build window
    let w_id = app
        .new_window()
        .size(1024, 1024)
        .title("nannou")
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id).unwrap(); // window reference
    let camera_is_active = true;
    let device = window.swap_chain_device(); // reference of gpu device
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let (win_w, win_h) = window.inner_size_pixels(); // size of screen

    // Load shader modules.
    // These modules are currently compiled in build.rs and should be SPIR-V format
    // $ glslangValidator -V shader.vert
    // $ glslangValidator -V shader.frag
    let vs_mod =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.vert.spv"));
    let fs_mod =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("../shaders/shader.frag.spv"));

    // Get reference of the vertex, normal and index buffere data
    let vertices_bytes = graphics::vertices_as_bytes(&data::VERTICES);
    let normals_bytes = graphics::normals_as_bytes(&data::NORMALS);
    let indices_bytes = graphics::indices_as_bytes(&data::INDICES);
    // Create the vertex, normal and index buffers
    let vertex_buffer = device.create_buffer_with_data(vertices_bytes, wgpu::BufferUsage::VERTEX);
    let normal_buffer = device.create_buffer_with_data(normals_bytes, wgpu::BufferUsage::VERTEX);
    let index_buffer = device.create_buffer_with_data(indices_bytes, wgpu::BufferUsage::INDEX);

    // create depth texture
    let depth_texture = graphics::create_depth_texture(
        device,
        [win_w, win_h],
        graphics::DEPTH_FORMAT,
        msaa_samples,
    );
    let depth_texture_view = depth_texture.view().build();

    // create camera struct
    let eye = Point3::new(0.0, 0.0, 1.0);
    let pitch = 0.0;
    let yaw = std::f32::consts::PI * 0.5;
    let camera = Camera { eye, pitch, yaw };

    // create uniform buffer
    let uniforms = graphics::create_uniforms([win_w, win_h], camera.view());
    let uniforms_bytes = graphics::uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST;
    let uniform_buffer = device.create_buffer_with_data(uniforms_bytes, usage);

    // Create the render pipeline.
    let bind_group_layout = graphics::create_bind_group_layout(device);
    let bind_group = graphics::create_bind_group(device, &bind_group_layout, &uniform_buffer);
    let pipeline_layout = graphics::create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = graphics::create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        graphics::DEPTH_FORMAT,
        msaa_samples,
    );

    let graphics = RefCell::new(Graphics {
        vertex_buffer,
        normal_buffer,
        index_buffer,
        uniform_buffer,
        depth_texture,
        depth_texture_view,
        bind_group,
        render_pipeline,
    });

    println!("Use the `W`, `A`, `S`, `D`, `Q` and `E` keys to move the camera.");
    println!("Use the mouse to orient the pitch and yaw of the camera.");
    println!("Press the `Space` key to toggle camera mode.");

    let is_capturing = false;
    let capture_begin_frame = 0u64;

    Model {
        camera_is_active,
        is_capturing,
        capture_begin_frame,
        graphics,
        camera,
    }
}

// Move the camera based on the current key pressed and its current direction.
fn update(_app: &App, _model: &mut Model, _update: Update) {
    // const CAM_SPEED_HZ: f64 = 0.5;
    // if model.camera_is_active {
    //     let velocity = (update.since_last.secs() * CAM_SPEED_HZ) as f32;
    //     // Go forwards on W.
    //     if app.keys.down.contains(&Key::W) {
    //         model.camera.eye += model.camera.direction() * velocity;
    //     }
    //     // Go backwards on S.
    //     if app.keys.down.contains(&Key::S) {
    //         model.camera.eye -= model.camera.direction() * velocity;
    //     }
    //     // Strafe left on A.
    //     if app.keys.down.contains(&Key::A) {
    //         let pitch = 0.0;
    //         let yaw = model.camera.yaw + std::f32::consts::PI * 0.5;
    //         let direction = pitch_yaw_to_direction(pitch, yaw);
    //         model.camera.eye += direction * velocity;
    //     }
    //     // Strafe right on D.
    //     if app.keys.down.contains(&Key::D) {
    //         let pitch = 0.0;
    //         let yaw = model.camera.yaw - std::f32::consts::PI * 0.5;
    //         let direction = pitch_yaw_to_direction(pitch, yaw);
    //         model.camera.eye += direction * velocity;
    //     }
    //     // Float down on Q.
    //     if app.keys.down.contains(&Key::Q) {
    //         let pitch = model.camera.pitch - std::f32::consts::PI * 0.5;
    //         let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
    //         model.camera.eye += direction * velocity;
    //     }
    //     // Float up on E.
    //     if app.keys.down.contains(&Key::E) {
    //         let pitch = model.camera.pitch + std::f32::consts::PI * 0.5;
    //         let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
    //         model.camera.eye += direction * velocity;
    //     }
    // }
}

// Use raw device motion event for camera pitch and yaw.
// TODO: Check device ID for mouse here - not sure if possible with winit currently.
fn event(_app: &App, _model: &mut Model, _event: Event) {
    // if model.camera_is_active {
    //     if let Event::DeviceEvent(_device_id, event) = event {
    //         if let winit::event::DeviceEvent::Motion { axis, value } = event {
    //             let sensitivity = 0.004;
    //             match axis {
    //                 // Yaw left and right on mouse x axis movement.
    //                 0 => model.camera.yaw -= (value * sensitivity) as f32,
    //                 // Pitch up and down on mouse y axis movement.
    //                 _ => {
    //                     let max_pitch = std::f32::consts::PI * 0.5 - 0.0001;
    //                     let min_pitch = -max_pitch;
    //                     model.camera.pitch = (model.camera.pitch + (-value * sensitivity) as f32)
    //                         .min(max_pitch)
    //                         .max(min_pitch)
    //                 }
    //             }
    //         }
    //     }
    // }
}

// Toggle cursor grabbing and hiding on Space key.
fn key_pressed(app: &App, model: &mut Model, key: Key) {
    // if let Key::Space = key {
    //     let window = app.main_window();
    //     if !model.camera_is_active {
    //         if window.set_cursor_grab(true).is_ok() {
    //             model.camera_is_active = true;
    //         }
    //     } else {
    //         if window.set_cursor_grab(false).is_ok() {
    //             model.camera_is_active = false;
    //         }
    //     }
    //     window.set_cursor_visible(!model.camera_is_active);
    // }
    if let Key::Space = key {
        model.is_capturing = !model.is_capturing;
        if model.is_capturing {
            if model.capture_begin_frame == 0 {
                model.capture_begin_frame = app.elapsed_frames();
            }
            println!("capture start");
        } else {
            println!("capture finished");
        }
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    let mut g = model.graphics.borrow_mut();

    // If the window has changed size, recreate our depth texture to match.
    let depth_size = g.depth_texture.size();
    let frame_size = frame.texture_size();
    let device = frame.device_queue_pair().device();
    if frame_size != depth_size {
        let depth_format = g.depth_texture.format();
        let sample_count = frame.texture_msaa_samples();
        g.depth_texture =
            graphics::create_depth_texture(device, frame_size, depth_format, sample_count);
        g.depth_texture_view = g.depth_texture.view().build();
    }

    // Update the uniforms (rotate around the teapot).
    let uniforms = graphics::create_uniforms(frame_size, model.camera.view());
    let uniforms_size = std::mem::size_of::<graphics::Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = graphics::uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsage::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_with_data(uniforms_bytes, usage);

    // create actual gpu commands (RenderPass)
    let mut encoder = frame.command_encoder();
    encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        // We'll use a depth texture to assist with the order of rendering fragments based on depth.
        .depth_stencil_attachment(&g.depth_texture_view, |depth| depth)
        .begin(&mut encoder);

    render_pass.set_bind_group(0, &g.bind_group, &[]); // set uniforms
    render_pass.set_pipeline(&g.render_pipeline);
    render_pass.set_vertex_buffer(0, &g.vertex_buffer, 0, 0);
    render_pass.set_vertex_buffer(1, &g.normal_buffer, 0, 0);
    render_pass.set_index_buffer(&g.index_buffer, 0, 0);

    // draw
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..1;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);

    // Capture the frame!
    if model.is_capturing {
        let num_capture_frame = app.elapsed_frames() - model.capture_begin_frame;
        let file_path = captured_frame_path(app, num_capture_frame);
        app.main_window().capture_frame(file_path);
    }
}

// conversion to movie file looks like:
// ffmpeg -framerate 30 -i "%5d.png" -pix_fmt yuv420p output.mp4
fn captured_frame_path(app: &App, num_frame: u64) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        .join("capture")
        // Name each file after the number of the frame.
        .join(format!("{:05}", num_frame))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}

// impl draw::Draw {
//     /// Render the **Draw**'s inner list of commands to the texture associated with the **Frame**.
//     ///
//     /// The **App** stores a unique render.
//     pub fn to_frame(&self, app: &App, frame: &Frame) -> Result<(), draw::renderer::DrawError> {
//         let window_id = frame.window_id();
//         let window = app
//             .window(window_id)
//             .expect("no window to draw to for `Draw`'s window_id");

//         // Retrieve a renderer for this window.
//         let renderers = app.draw_state.renderers.borrow_mut();
//         let renderer = RefMut::map(renderers, |renderers| {
//             renderers.entry(window_id).or_insert_with(|| {
//                 let device = window.swap_chain_device();
//                 let frame_dims: [u32; 2] = window.tracked_state.physical_size.into();
//                 let scale_factor = window.tracked_state.scale_factor as f32;
//                 let msaa_samples = window.msaa_samples();
//                 let target_format = crate::frame::Frame::TEXTURE_FORMAT;
//                 let renderer = draw::RendererBuilder::new().build(
//                     device,
//                     frame_dims,
//                     scale_factor,
//                     msaa_samples,
//                     target_format,
//                 );
//                 RefCell::new(renderer)
//             })
//         });

//         let scale_factor = window.tracked_state.scale_factor as _;
//         let mut renderer = renderer.borrow_mut();
//         renderer.render_to_frame(window.swap_chain_device(), self, scale_factor, frame);
//         Ok(())
//     }
// }
