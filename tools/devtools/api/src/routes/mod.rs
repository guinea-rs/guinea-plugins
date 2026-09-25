//! Every route, and nothing else: what each one answers is the model's.

mod apps;
mod drive;
mod native;
mod trace;

use guinea_devtools_model::sessions::{Session, Sessions};
use utoipa_axum::router::OpenApiRouter;

use crate::Failure;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .merge(apps::router())
        .merge(trace::router())
        .merge(native::router())
        .merge(drive::router())
}

/// The session `app` names: an id, or `latest` for the newest one still
/// connected.
fn session<'a>(sessions: &'a Sessions, app: &str) -> Result<&'a Session, Failure> {
    let found = if app == "latest" {
        sessions.newest().or_else(|| sessions.by_id.values().next_back())
    } else {
        app.parse().ok().and_then(|id| sessions.get(id))
    };

    found.ok_or_else(|| Failure::not_found(format!("no application {app}")))
}

/// A comma-separated query value, which is how a list arrives.
fn list(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}
