use guinea::feature::Install;
use guinea::{feature, installs};
use guinea_devtools_model::chains::Stream;
use guinea_plugin_store::{Store, StoreAccess};

use super::actor::TraceActor;
use super::contracts::TraceState;
use super::settings::TraceSettings;

feature! {
    pub TraceFeature {
        exports { TraceState }
    }
}

#[installs]
fn trace(cx: &Install, store: Store) -> anyhow::Result<TraceFeature> {
    let settings = store.try_settings::<TraceSettings>()?;

    let seed = TraceState {
        stream: settings.stream().get().parse().unwrap_or(Stream::All),
        query: settings.query().get(),
        ..TraceState::default()
    };

    let (state, _) = cx
        .state::<TraceState>()
        .seed(seed)
        .driven_by(|push| TraceActor::new(push, settings));

    Ok(TraceFeature(state))
}
