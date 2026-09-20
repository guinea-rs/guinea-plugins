//! The backend's own elements, through whatever inspector the application
//! has. Reading is a `GET`; asking the inspector to do something is a `POST`,
//! and what it answers shows up in the next read.

use axum::Json;
use axum::extract::{Path, Query, State};
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::protocol::native::{Bounds, Frame, Property};
use guinea_devtools_model::sessions::Session;
use guinea_devtools_protocol::Command;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::session;
use crate::Failure;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .routes(routes!(native))
        .routes(routes!(attach))
        .routes(routes!(properties))
        .routes(routes!(hit))
        .routes(routes!(highlight))
        .routes(routes!(set))
        .routes(routes!(perf))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Cut {
    /// Where to start, instead of every root.
    root: Option<u64>,
    /// How many levels below that to include.
    depth: Option<usize>,
}

/// The native tree, with whatever the inspector last answered.
#[utoipa::path(
    get,
    path = "/apps/{app}/native",
    tag = "native",
    operation_id = "get_native_tree",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Cut),
    responses(
        (status = 200, description = "The tree, the last properties, pick and refusal"),
        (status = 404, description = "The application has no inspector: attach one first")
    )
)]
async fn native(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(cut): Query<Cut>,
) -> Result<Json<View>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let inspector = sessions
            .native_for(session.id)
            .ok_or_else(|| Failure::not_found("no native inspector: attach one first"))?;

        Ok(Json(View::of(inspector, cut.root, cut.depth.unwrap_or(6))))
    })
}

/// Loads the XAML tap into the application, if it has none.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/attach",
    tag = "native",
    operation_id = "attach_native_inspector",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses(
        (status = 200, description = "The tap was loaded, or was already there"),
        (status = 409, description = "It could not be loaded"),
        (status = 501, description = "Not this platform")
    )
)]
async fn attach(State(state): State<crate::State>, Path(app): Path<String>) -> Result<Json<Done>, Failure> {
    let id = state.read(|sessions| session(sessions, &app).map(|session| session.id))?;

    state.attach(id).map_err(Failure::refused)?;

    Ok(Json(Done { done: true }))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct One {
    /// The element, as the tree names it.
    element: u64,
}

/// Asks for one element's properties.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/properties",
    tag = "native",
    operation_id = "ask_native_properties",
    params(("app" = String, Path, description = "An application's id, or `latest`"), One),
    responses(
        (status = 200, description = "Asked; read `/native` for the answer"),
        (status = 404, description = "The application has no inspector")
    )
)]
async fn properties(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(one): Query<One>,
) -> Result<Json<Done>, Failure> {
    command(&state, &app, Command::NativeProperties { element: one.element })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct At {
    /// Screen pixels.
    x: i32,
    y: i32,
}

/// Asks what is under a point on the screen.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/hit",
    tag = "native",
    operation_id = "native_hit_test",
    params(("app" = String, Path, description = "An application's id, or `latest`"), At),
    responses(
        (status = 200, description = "Asked; the chain shows up in `/native`"),
        (status = 404, description = "The application has no inspector")
    )
)]
async fn hit(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(at): Query<At>,
) -> Result<Json<Done>, Failure> {
    command(&state, &app, Command::NativeHitTest { x: at.x, y: at.y })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Maybe {
    /// The element to outline; left out, the outline goes away.
    element: Option<u64>,
}

/// Outlines an element in the application's own window.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/highlight",
    tag = "native",
    operation_id = "highlight_native_element",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Maybe),
    responses(
        (status = 200, description = "Asked"),
        (status = 404, description = "The application has no inspector")
    )
)]
async fn highlight(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(maybe): Query<Maybe>,
) -> Result<Json<Done>, Failure> {
    command(&state, &app, Command::NativeHighlight { element: maybe.element })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Write {
    element: u64,
    /// The property, by the index `/native` gives it.
    property: u32,
    /// Its type, as the property says it is.
    #[serde(rename = "type")]
    type_name: String,
    /// The new value, written the way the backend writes one.
    value: String,
}

/// Changes one property, live.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/set",
    tag = "native",
    operation_id = "set_native_property",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Write),
    responses(
        (status = 200, description = "Asked; the new properties show up in `/native`"),
        (status = 404, description = "The application has no inspector")
    )
)]
async fn set(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(write): Query<Write>,
) -> Result<Json<Done>, Failure> {
    command(
        &state,
        &app,
        Command::NativeSetProperty {
            element: write.element,
            property: write.property,
            type_name: write.type_name,
            value: write.value,
        },
    )
}

/// Captures the frames the backend recorded lately.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/perf",
    tag = "native",
    operation_id = "capture_native_frames",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses(
        (status = 200, description = "Asked; the frames show up in `/native`"),
        (status = 404, description = "The application has no inspector")
    )
)]
async fn perf(State(state): State<crate::State>, Path(app): Path<String>) -> Result<Json<Done>, Failure> {
    command(&state, &app, Command::NativePerfCapture)
}

/// Sends `command` to the inspector of `app`.
fn command(state: &crate::State, app: &str, command: Command) -> Result<Json<Done>, Failure> {
    let inspector = state.read(|sessions| {
        let session = session(sessions, app)?;

        sessions
            .native_for(session.id)
            .map(|inspector| inspector.id)
            .ok_or_else(|| Failure::not_found("no native inspector: attach one first"))
    })?;

    state.send(inspector, command).map_err(Failure::refused)?;

    Ok(Json(Done { done: true }))
}

/// What a command answers: it was queued, and what it did shows up in the
/// next read.
#[derive(Serialize)]
struct Done {
    done: bool,
}

/// The native tree cut to a depth, and what the inspector answered last.
#[derive(Serialize)]
struct View {
    /// The inspector's own session id.
    session: u64,
    elements: usize,
    /// How many enumerations the inspector sent, and how many values in all.
    enums: (usize, usize),
    tree: Vec<Node>,
    properties: Option<(u64, Vec<Property>)>,
    picked: Option<Vec<Node>>,
    /// Where the innermost picked element is on the screen.
    picked_bounds: Option<Bounds>,
    /// The last command the inspector would not carry out, and why.
    refused: Option<(String, String)>,
    /// The frames of the last capture.
    frames: Vec<Frame>,
}

#[derive(Serialize)]
struct Node {
    handle: u64,
    kind: String,
    name: String,
    children: Vec<Node>,
    /// Children left out below the depth asked for.
    hidden: usize,
}

impl View {
    fn of(inspector: &Session, root: Option<u64>, depth: usize) -> View {
        let tree = &inspector.inspection.tree;
        let starts: Vec<u64> = root.map_or_else(|| tree.roots().to_vec(), |root| vec![root]);

        View {
            session: inspector.id,
            elements: tree.len(),
            enums: (
                inspector.inspection.enums.len(),
                inspector.inspection.enums.values().map(Vec::len).sum(),
            ),
            tree: starts.iter().filter_map(|&handle| node(tree, handle, depth)).collect(),
            properties: inspector.inspection.properties.clone(),
            picked: inspector
                .inspection
                .picked
                .as_ref()
                .map(|picked| picked.chain.iter().filter_map(|&handle| node(tree, handle, 0)).collect()),
            picked_bounds: inspector.inspection.picked.as_ref().and_then(|picked| picked.bounds),
            refused: inspector.inspection.refused.clone(),
            frames: inspector.inspection.frames.clone(),
        }
    }
}

fn node(tree: &NativeTree, handle: u64, depth: usize) -> Option<Node> {
    let element = tree.get(handle)?;
    let children = tree.children(handle);

    Some(Node {
        handle,
        kind: element.kind.clone(),
        name: element.name.clone(),
        children: if depth == 0 {
            Vec::new()
        } else {
            children.iter().filter_map(|&child| node(tree, child, depth - 1)).collect()
        },
        hidden: if depth == 0 { children.len() } else { 0 },
    })
}
