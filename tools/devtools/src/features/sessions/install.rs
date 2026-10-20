use guinea::feature::Install;
use guinea::{feature, installs};

use super::actor::SessionsActor;
use super::contracts::{Listen, Live};

feature! {
    pub SessionsFeature {
        exports { Live }
    }
}

#[installs]
fn sessions(cx: &Install) -> anyhow::Result<SessionsFeature> {
    let (live, actor) = cx.state::<Live>().driven_by(SessionsActor::new);
    actor.send(Listen);
    Ok(SessionsFeature(live))
}
