use nannou::prelude::*;

pub type BufferSize = std::num::NonZeroU64;

mod camera;
mod geom;
mod light;
mod pass;
mod texture;

struct Model {
    pass: pass::Pass,
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
        .resized(resized)
        .raw_view(raw_view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(window_id).unwrap(); // window reference
    let device = window.swap_chain_device(); // reference of gpu device
    let queue = window.swap_chain_queue();
    let sc_desc = window.swap_chain_descriptor();

    let pass = pass::Pass::new(device, queue, sc_desc);

    Model { pass }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let queue = window.swap_chain_queue();

    model.pass.update(queue);
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn resized(app: &App, model: &mut Model, _: Vector2) {
    let window = app.main_window();
    let device = window.swap_chain_device();
    let sc_desc = window.swap_chain_descriptor();

    model.pass.resized(device, sc_desc);
}

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {
    // if let Key::Space = key {
    //     model.is_capturing = !model.is_capturing;
    //     if model.is_capturing {
    //         if model.capture_begin_frame == 0 {
    //             model.capture_begin_frame = app.elapsed_frames();
    //         }
    //         println!("capture start");
    //     } else {
    //         println!("capture finished");
    //     }
    // }
}

fn raw_view(app: &App, model: &Model, raw_frame: RawFrame) {
    let window = app.main_window();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    model
        .pass
        .draw(&mut encoder, raw_frame.swap_chain_texture());

    queue.submit(std::iter::once(encoder.finish()));

    // // TODO:
    // if model.is_capturing {
    //     let num_capture_frame = app.elapsed_frames() - model.capture_begin_frame;
    //     let file_path = app
    //         .project_path()
    //         .unwrap()
    //         .join(format!("{:05}", num_capture_frame))
    //         .with_extension("png");
    //     app.main_window().capture_frame(file_path);
    // }
}

// // conversion to movie file looks like:
// // ffmpeg -framerate 30 -i "%5d.png" -pix_fmt yuv420p output.mp4
// fn captured_frame_path(app: &App, num_frame: u64) -> std::path::PathBuf {}
