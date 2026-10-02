use guinea::feature::FeatureInitContext;
use guinea_macros::{feature, installs};
use guinea_plugin_store::StoreAccess;

use super::actor::PanelsActor;
use super::contracts::PanelsState;
use super::settings::PanelsSettings;

feature! {
    pub PanelsFeature {
        exports { PanelsState }
    }
}

#[installs]
fn panels(cx: &FeatureInitContext) -> anyhow::Result<PanelsFeature> {
    let settings = cx.settings::<PanelsSettings>()?;
    let panel = settings.panel().get();

    let seed = PanelsState {
        node: vec![settings.section().get()],
        panel: (!panel.is_empty()).then_some(panel),
    };

    let (state, _) = cx
        .state::<PanelsState>()
        .seed(seed)
        .driven_by(|push| PanelsActor::new(push, settings));

    Ok(PanelsFeature(state))
}
