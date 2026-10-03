mod components;
mod features;
mod fonts;
mod icons;
mod layouts;
mod pages;
mod routes;
mod theme;

use clap::Parser;
use guinea::app::GuineaApp;
use guinea::eframe::run;
use guinea_devtools_protocol::IDENTIFIER;
use guinea_plugin_single_instance::SingleInstancePlugin;
use routes::Route;

/// Looks inside running guinea applications.
#[derive(Parser)]
#[command(version, after_long_help = api())]
struct Arguments {
    /// Serve the HTTP API without opening a window.
    #[arg(long)]
    headless: bool,
}

/// The HTTP API, as its own OpenAPI document describes it.
fn api() -> String {
    guinea_devtools_api::help::routes(&guinea_devtools_api::document())
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    if arguments.headless {
        let Some(_instance) = guinea_plugin_single_instance::claim(IDENTIFIER)? else {
            println!("guinea devtools are already running");
            return Ok(());
        };

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
            IDENTIFIER,
            env!("CARGO_PKG_VERSION"),
            "uniproc",
        ))
        .plugin(SingleInstancePlugin::new())
        .plugin(
            guinea_plugin_store::StorePlugin::for_app("guinea-devtools", "settings")
                .backend(amethystate::store::builder::Backend::Json),
        );

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/guinea.png"))?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_title("guinea devtools")
            .with_icon(icon),
        ..Default::default()
    };

    run(app, "guinea devtools", options, |_| Route::Home {})
}
