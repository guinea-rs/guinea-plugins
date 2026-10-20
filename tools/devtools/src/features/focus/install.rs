use guinea::feature::Install;
use guinea::{feature, installs};

use super::actor::FocusActor;
use super::contracts::{Change, Focus};

feature! {
    pub FocusFeature {
        exports { Focus }
    }
}

#[installs]
fn focus(cx: &Install<u64>) -> anyhow::Result<FocusFeature> {
    let app = cx.params();
    let seed = Focus {
        app: *app,
        ..Focus::default()
    };

    let (focus, _) = cx.state::<Focus>().seed(seed).driven_by(FocusActor::new);
    focus.push(Change::App(*app));

    Ok(FocusFeature(focus))
}
