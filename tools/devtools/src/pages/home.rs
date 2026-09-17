use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;

use crate::sessions::contracts::Live;
use crate::style;

pub struct Home;

impl Page for Home {
    type Params = crate::routes::HomeParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let ui = cx.ui();
        let listening = live.read().listening.describe();

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Waiting for an application");
            ui.add_space(8.0);
            ui.label(style::dim(
                "An application shows up here once it installs DevToolsPlugin.",
            ));
            ui.label(style::dim(listening));
        });
    }
}
