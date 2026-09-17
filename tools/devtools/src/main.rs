mod demo;
mod editor;
mod focus;
mod fonts;
mod icons;
mod layouts;
mod memory;
mod pages;
mod routes;
mod sessions;
mod style;
mod theme;
mod trace_view;

use clap::Parser;
use guinea::app::GuineaApp;
use guinea::eframe::run;
use routes::Route;

/// Looks inside running guinea applications.
#[derive(Parser)]
#[command(version)]
struct Arguments {
    /// Serve the HTTP API without opening a window.
    #[arg(long)]
    headless: bool,
    /// Connect two made-up applications, to have something to look at.
    #[arg(long)]
    demo: bool,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    if arguments.demo {
        demo::spawn();
    }

    if arguments.headless {
        let hub = guinea_devtools_hub::Hub::start();
        let access = guinea_devtools_hub::serve(hub)?;
        println!("guinea devtools, without a window: {}", access.url);

        loop {
            std::thread::park();
        }
    }

    let app = GuineaApp::new()
        .meta(guinea::app::AppMeta::new(
            "guinea devtools",
            "dev.uniproc.guinea.devtools",
            env!("CARGO_PKG_VERSION"),
            "uniproc",
        ))
        .plugin(
            guinea_plugin_store::StorePlugin::for_app("guinea-devtools", "settings")
                .backend(amethystate::store::builder::Backend::Json),
        )
        .plugin(memory::MemoryPlugin);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_title("guinea devtools"),
        ..Default::default()
    };

    run(app, "guinea devtools", options, || Route::Home {})
}
