use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    is_capturing: bool,
    capture_begin_frame: u64,
}

fn model(app: &App) -> Model {
    let _win_id = app
        .new_window()
        .size(1280, 720)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    Model {
        is_capturing: false,
        capture_begin_frame: 0u64,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
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

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();

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
