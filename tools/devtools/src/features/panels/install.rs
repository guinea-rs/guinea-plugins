use guinea::feature::Install;
use guinea::{feature, installs};
use guinea_plugin_store::{Store, StoreAccess};

use super::actor::PanelsActor;
use super::contracts::PanelsState;
use super::settings::PanelsSettings;

feature! {
    pub PanelsFeature {
        exports { PanelsState }
    }
}

#[installs]
fn panels(cx: &Install, store: Store) -> anyhow::Result<PanelsFeature> {
    let settings = store.try_settings::<PanelsSettings>()?;
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
