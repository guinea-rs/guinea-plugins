use guinea_macros::routes;

use crate::layouts::app::App;
use crate::layouts::shell::Shell;
use crate::pages::application::Application;
use crate::pages::elements::Elements;
use crate::pages::home::Home;
use crate::pages::profiler::Profiler;
use crate::pages::trace::Traces;

routes! {
    backend = guinea::eframe::Egui,
    Route {
        layout(Shell) {
            page(Home)
            layout(App) {
                page(Elements) { app: u64 }
                page(Traces) { app: u64 }
                page(Profiler) { app: u64 }
                page(Application) { app: u64 }
            }
        }
    }
}
