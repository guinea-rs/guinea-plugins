use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::SessionsActor;
use super::contracts::{Listen, Live};

pub struct SessionsFeature {
    _live: Bound<Live>,
}

#[installs]
impl Feature for SessionsFeature {
    type Exports = (Live,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let (live, _) = cx.state::<Live>().driven_by(SessionsActor::new);
        live.emit(Listen);
        Ok(Self { _live: live })
    }
}
