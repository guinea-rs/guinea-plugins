use guinea::feature::FeatureInitContext;
use guinea_macros::{feature, installs};
use guinea_plugin_store::StoreAccess;

use super::actor::LastTabActor;
use super::contracts::LastTab;
use super::settings::TabSettings;

feature! {
    pub LastTabFeature {
        exports { LastTab }
    }
}

#[installs]
fn last_tab(cx: &FeatureInitContext) -> anyhow::Result<LastTabFeature> {
    let settings = cx.settings::<TabSettings>()?;
    let seed = LastTab(settings.tab().get());

    let (tab, _) = cx
        .state::<LastTab>()
        .seed(seed)
        .driven_by(|push| LastTabActor::new(push, settings));

    Ok(LastTabFeature(tab))
}
