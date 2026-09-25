use guinea::feature::FeatureInitContext;
use guinea_macros::{feature, installs};

use super::actor::SessionsActor;
use super::contracts::{Listen, Live};

feature! {
    pub SessionsFeature {
        exports { Live }
    }
}

#[installs]
fn sessions(cx: &FeatureInitContext) -> anyhow::Result<SessionsFeature> {
    let (live, _) = cx.state::<Live>().driven_by(SessionsActor::new);
    live.emit(Listen);
    Ok(SessionsFeature(live))
}
