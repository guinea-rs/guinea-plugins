//! Icons from `icons.gui.toml`, drawn white and tinted to the text around
//! them.

use egui::ImageSource;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::words::Level;

pub fn once(ctx: &egui::Context) {
    let done = egui::Id::new("image-loaders");
    if ctx.data(|data| data.get_temp::<bool>(done)).is_none() {
        ctx.data_mut(|data| data.insert_temp(done, true));
        egui_extras::install_image_loaders(ctx);
    }
}

/// An icon the size of a line of text, in `color`.
pub fn image(
    source: ImageSource<'static>,
    size: f32,
    color: egui::Color32,
) -> egui::Image<'static> {
    egui::Image::new(source)
        .fit_to_exact_size(egui::vec2(size, size))
        .tint(color)
}

/// What a stream stands for.
pub fn stream(stream: &Stream) -> ImageSource<'static> {
    match stream {
        Stream::All => guicons::icon!(everything),
        Stream::Records => guicons::icon!(records),
        Stream::Level(Level::Error) => guicons::icon!(error),
        Stream::Level(Level::Warn) => guicons::icon!(warn),
        Stream::Level(Level::Info) => guicons::icon!(info),
        Stream::Level(Level::Debug) => guicons::icon!(debug),
        Stream::Level(Level::Trace) => guicons::icon!(trace),
        Stream::Slow => guicons::icon!(slow),
        Stream::Action(_) => guicons::icon!(action),
        Stream::Timer(_) => guicons::icon!(timer),
        Stream::Source(_) => guicons::icon!(source),
        Stream::Loop(_) => guicons::icon!(repeat),
        Stream::Navigation => guicons::icon!(navigation),
        Stream::Store => guicons::icon!(store),
        Stream::Log => guicons::icon!(log),
        Stream::Loose => guicons::icon!(loose),
    }
}

pub fn close() -> ImageSource<'static> {
    guicons::icon!(close)
}

pub fn search() -> ImageSource<'static> {
    guicons::icon!(search)
}
