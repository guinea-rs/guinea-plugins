//! The guinea pig that runs while devtools wait for an application.

use std::time::Duration;

use egui::{ColorImage, Image, Rect, TextureHandle, TextureOptions, pos2, vec2};

const FRAMES: usize = 4;
const FPS: f64 = 10.0;
const SIZE: f32 = 64.0;

/// The run cycle as one texture, four frames side by side. Uploaded once and
/// kept by the page that draws it.
pub fn run_cycle(ctx: &egui::Context) -> TextureHandle {
    let sheet = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/guinea-run.png"))
        .expect("the run cycle ships inside the binary");
    let image = ColorImage::from_rgba_unmultiplied([sheet.width as usize, sheet.height as usize], &sheet.rgba);

    ctx.load_texture("guinea-run", image, TextureOptions::NEAREST)
}

/// The frame of the run cycle for now, with a repaint booked for the next.
pub fn runner(ui: &mut egui::Ui, cycle: &TextureHandle) -> egui::Response {
    let time = ui.input(|input| input.time);
    let frame = (time * FPS) as usize % FRAMES;

    let step = 1.0 / FRAMES as f32;
    let uv = Rect::from_min_max(
        pos2(frame as f32 * step, 0.0),
        pos2((frame + 1) as f32 * step, 1.0),
    );

    let period = 1.0 / FPS;
    ui.ctx()
        .request_repaint_after(Duration::from_secs_f64(period - time % period));

    ui.add(Image::from_texture((cycle.id(), vec2(SIZE, SIZE))).uv(uv))
}
