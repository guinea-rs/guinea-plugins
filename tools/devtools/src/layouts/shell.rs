use guinea::eframe::{Layout, LayoutCx};
use guinea::feature::FeatureInitContext;

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

impl Layout for Shell {
    type Params = crate::routes::ShellParams;
    type Installs = (SessionsFeature, LastTabFeature, EditorFeature);

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok((ctx.install(&())?, ctx.install(&())?, ctx.install(&())?))
    }

    fn render(&mut self, cx: &mut LayoutCx<'_, Self>) {
        let (live, _) = cx.read::<Live>();
        let (last, _) = cx.read::<LastTab>();
        let nav = cx.navigate::<Route>();
        let waiting = cx.child_is::<Home>();
        let page = cx.outlet();
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
