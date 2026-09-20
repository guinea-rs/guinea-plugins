use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::LastTabActor;
use super::contracts::LastTab;
use super::settings::TabSettings;

pub struct LastTabFeature {
    _tab: Bound<LastTab>,
}

#[installs]
impl Feature for LastTabFeature {
    type Exports = (LastTab,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let settings = TabSettings::new()?;
        let seed = LastTab(settings.tab().get());

        let (tab, _) = cx
            .state::<LastTab>()
            .seed(seed)
            .driven_by(|push| LastTabActor::new(push, settings));

        Ok(Self { _tab: tab })
    }
}
