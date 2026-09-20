use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::NativeActor;
use super::contracts::NativeState;

pub struct NativeFeature {
    _state: Bound<NativeState>,
}

#[installs]
impl Feature for NativeFeature {
    type Exports = (NativeState,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let (state, _) = cx.state::<NativeState>().driven_by(NativeActor::new);
        Ok(Self { _state: state })
    }
}
