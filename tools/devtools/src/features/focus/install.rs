use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::FocusActor;
use super::contracts::{Change, Focus};

pub struct FocusFeature {
    _focus: Bound<Focus>,
}

#[installs]
impl Feature for FocusFeature {
    type Exports = (Focus,);

    fn install(cx: &FeatureInitContext, app: &u64) -> anyhow::Result<Self> {
        let seed = Focus {
            app: *app,
            ..Focus::default()
        };

        let (focus, _) = cx.state::<Focus>().seed(seed).driven_by(FocusActor::new);
        focus.push(Change::App(*app));

        Ok(Self { _focus: focus })
    }
}
