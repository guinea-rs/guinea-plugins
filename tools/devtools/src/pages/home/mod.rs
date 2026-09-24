use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;

use crate::components;
use crate::features::sessions::contracts::Live;

#[derive(Default)]
pub struct Home {
    cycle: Option<egui::TextureHandle>,
}

impl Page for Home {
    type Params = crate::routes::HomeParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let ui = cx.ui();
        let listening = live.read().listening.describe();
        let cycle = self.cycle.get_or_insert_with(|| components::run_cycle(ui.ctx()));

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            components::runner(ui, cycle);
            ui.add_space(12.0);
            ui.heading("Waiting for an application");
            ui.add_space(8.0);
            ui.label(components::dim(
                "An application shows up here once it installs DevToolsPlugin.",
            ));
            ui.label(components::dim(listening));
        });
    }
}
