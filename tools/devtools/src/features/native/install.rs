use guinea::feature::FeatureInitContext;
use guinea_macros::{feature, installs};

use super::actor::NativeActor;
use super::contracts::NativeState;

feature! {
    pub NativeFeature {
        exports { NativeState }
    }
}

#[installs]
fn native(cx: &FeatureInitContext) -> anyhow::Result<NativeFeature> {
    let (state, _) = cx.state::<NativeState>().driven_by(NativeActor::new);
    Ok(NativeFeature(state))
}
