use guinea::eframe::{Layout, LayoutCx};
use guinea::feature::Install;

use crate::features::editor::EditorFeature;
use crate::features::sessions::SessionsFeature;
use crate::features::sessions::contracts::Live;
use crate::features::tab::LastTabFeature;
use crate::features::tab::contracts::LastTab;
use crate::layouts::app::tab_named;
use crate::pages::home::Home;
use crate::routes::Route;

#[derive(Default)]
pub struct Shell;

#[guinea::eframe::layout]
impl Layout for Shell {
    type Params = ();

    fn install(cx: &Install) -> anyhow::Result<Self> {
        cx.install::<SessionsFeature>(&())?;
        cx.install::<LastTabFeature>(&())?;
        cx.install::<EditorFeature>(&())?;
        Ok(Self)
    }

    fn render(&mut self, cx: &mut LayoutCx<'_, Self>) {
        let (live, _) = cx.read::<Live>();
        let (last, _) = cx.read::<LastTab>();
        let nav = cx.navigate::<Route>();
        let waiting = matches!(nav.current(), Some(Route::Home {}));
        let page = cx.outlet(&nav, |route, cx| match route {
            Route::Home {} => cx.mount::<Home>(&()),
            route => cx.mount::<crate::layouts::app::App>(&crate::routes::AppParams {
                app: route.app().unwrap(),
            }),
        });
        let ui = cx.ui();

        crate::fonts::once(ui.ctx());
        crate::theme::once(ui.ctx());
        crate::icons::once(ui.ctx());

        let newest = live.read().newest().map(|session| session.id);
        if waiting && let Some(app) = newest {
            nav.to(tab_named(&last.0, app));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0))
            .show(ui, |ui| page.draw(ui));
    }
}
