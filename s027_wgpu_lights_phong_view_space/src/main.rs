use nannou::prelude::*;

pub type BufferSize = std::num::NonZeroU64;

mod pass;
mod renderer;

use crate::pass::PassMain;

struct Model {
    pass: pass::PassMain,
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

    let window = app.window(window_id).unwrap();
    let device = window.swap_chain_device();
    let queue = window.swap_chain_queue();
    let sc_desc = window.swap_chain_descriptor();

    let pass = PassMain::new(device, queue, sc_desc);

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

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {}

fn raw_view(_app: &App, model: &Model, raw_frame: RawFrame) {
    let mut encoder = raw_frame.command_encoder();
    model
        .pass
        .draw(&mut encoder, raw_frame.swap_chain_texture());
}
