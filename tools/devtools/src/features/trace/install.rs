use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::words::Kind;
use guinea_macros::installs;

use super::actor::TraceActor;
use super::contracts::TraceState;
use super::settings::TraceSettings;

pub struct TraceFeature {
    _state: Bound<TraceState>,
}

#[installs]
impl Feature for TraceFeature {
    type Exports = (TraceState,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let settings = TraceSettings::new()?;
        let hidden = settings.hidden().get();

        let seed = TraceState {
            stream: settings.stream().get().parse().unwrap_or(Stream::All),
            query: settings.query().get(),
            hidden: Kind::ALL
                .into_iter()
                .filter(|kind| hidden.iter().any(|named| named == kind.name()))
                .collect(),
            ..TraceState::default()
        };

        let (state, _) = cx
            .state::<TraceState>()
            .seed(seed)
            .driven_by(|push| TraceActor::new(push, settings));

        Ok(Self { _state: state })
    }
}
