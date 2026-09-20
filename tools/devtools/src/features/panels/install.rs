use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::PanelsActor;
use super::contracts::PanelsState;
use super::settings::PanelsSettings;

pub struct PanelsFeature {
    _state: Bound<PanelsState>,
}

#[installs]
impl Feature for PanelsFeature {
    type Exports = (PanelsState,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let settings = PanelsSettings::new()?;
        let panel = settings.panel().get();

        let seed = PanelsState {
            node: vec![settings.section().get()],
            panel: (!panel.is_empty()).then_some(panel),
        };

        let (state, _) = cx
            .state::<PanelsState>()
            .seed(seed)
            .driven_by(|push| PanelsActor::new(push, settings));

        Ok(Self { _state: state })
    }
}
