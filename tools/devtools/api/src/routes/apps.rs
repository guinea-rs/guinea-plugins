//! What connected, and what each application is made of.

use std::collections::HashSet;

use axum::Json;
use axum::extract::{Path, Query, State};
use guinea_devtools_model::elements::{self, Details, Element, Line};
use guinea_devtools_model::graph::{self, Graph};
use guinea_devtools_model::panels::{self, Listed};
use guinea_devtools_model::protocol::Snapshot;
use guinea_devtools_model::sessions::Summary;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::{list, session};
use crate::Failure;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .routes(routes!(apps))
        .routes(routes!(snapshot))
        .routes(routes!(elements))
        .routes(routes!(element))
        .routes(routes!(contributed))
        .routes(routes!(wiring))
}

/// Every application that connected, newest first.
#[utoipa::path(
    get,
    path = "/apps",
    tag = "apps",
    operation_id = "list_apps",
    responses((status = 200, description = "Every application, newest first"))
)]
async fn apps(State(state): State<crate::State>) -> Json<Vec<Summary>> {
    Json(state.read(|sessions| sessions.summaries()))
}

/// The last snapshot, as the application sent it.
#[utoipa::path(
    get,
    path = "/apps/{app}/snapshot",
    tag = "apps",
    operation_id = "get_snapshot",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses((status = 200, description = "Roots, actors, reducers, panels and timers"))
)]
async fn snapshot(
    State(state): State<crate::State>,
    Path(app): Path<String>,
) -> Result<Json<Snapshot>, Failure> {
    state.read(|sessions| Ok(Json(session(sessions, &app)?.snapshot.clone())))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Tree {
    /// Elements the other way from how they start, comma-separated ids: a
    /// row open by default closed, a native one - closed by default - open.
    #[serde(default)]
    flipped: String,
    /// An element to bring into view, opening whatever hides it.
    reveal: Option<String>,
}

/// The application as one tree: windows, segments, features, state, actors,
/// and what each segment made in the backend's own tree when an inspector is
/// attached.
#[utoipa::path(
    get,
    path = "/apps/{app}/elements",
    tag = "apps",
    operation_id = "get_elements",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Tree),
    responses((status = 200, description = "One line per row, with its depth and words"))
)]
async fn elements(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(tree): Query<Tree>,
) -> Result<Json<Vec<Line>>, Failure> {
    let flipped: HashSet<Element> = list(&tree.flipped)
        .iter()
        .map(|id| id.parse().map_err(Failure::bad_request))
        .collect::<Result<_, _>>()?;
    let reveal: Option<Element> = tree
        .reveal
        .map(|id| id.parse().map_err(Failure::bad_request))
        .transpose()?;

    state.read(|sessions| {
        let session = session(sessions, &app)?;

        let native = sessions
            .native_for(session.id)
            .map(|inspector| &inspector.inspection.tree);

        Ok(Json(elements::lines(
            &session.snapshot,
            native,
            &flipped,
            reveal.as_ref(),
        )))
    })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Which {
    /// `app`, `actor/7`, `window/1`, `segment/1/0`, `feature/1/0/TabsFeature`,
    /// `state/1/0/a::Tabs`, `view/1/0.2.1`. A native element, `native/2178`,
    /// is described by the native routes instead.
    id: String,
}

/// What is known about one element of that tree.
#[utoipa::path(
    get,
    path = "/apps/{app}/element",
    tag = "apps",
    operation_id = "get_element",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Which),
    responses(
        (status = 200, description = "Its rows, where it was declared, what it handles"),
        (status = 404, description = "No such element")
    )
)]
async fn element(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(which): Query<Which>,
) -> Result<Json<Details>, Failure> {
    let element: Element = which.id.parse().map_err(Failure::bad_request)?;

    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let details = elements::describe(session, &element)
            .ok_or_else(|| Failure::not_found(format!("no element {}", which.id)))?;

        Ok(Json(details))
    })
}

/// What plugins and backends contributed about themselves.
#[utoipa::path(
    get,
    path = "/apps/{app}/panels",
    tag = "apps",
    operation_id = "get_panels",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses((status = 200, description = "Each panel, with the key it is found by"))
)]
async fn contributed(
    State(state): State<crate::State>,
    Path(app): Path<String>,
) -> Result<Json<Vec<Listed>>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;

        Ok(Json(panels::listed(&session.snapshot)))
    })
}

/// Actors, reducers and buses, and what goes between them.
#[utoipa::path(
    get,
    path = "/apps/{app}/graph",
    tag = "apps",
    operation_id = "get_graph",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses((status = 200, description = "Nodes in clusters, and the edges seen so far"))
)]
async fn wiring(
    State(state): State<crate::State>,
    Path(app): Path<String>,
) -> Result<Json<Graph>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;

        Ok(Json(graph::build(&session.snapshot, &session.trace)))
    })
}
