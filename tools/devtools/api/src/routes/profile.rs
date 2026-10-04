//! Where the time of a frame went: seconds of what the inspector last
//! captured, the frames of one, and one frame with what the application
//! recorded towards it. Capturing is `POST /apps/{app}/native/perf`.

use axum::Json;
use axum::extract::{Path, State};
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::profile::{BUDGET_US, Frame, Profile};
use guinea_devtools_model::protocol::native::Pass;
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_model::words;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::session;
use crate::Failure;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .routes(routes!(seconds))
        .routes(routes!(frames))
        .routes(routes!(frame))
}

/// The application whose trace the frames go on, its profile, and the tree
/// its inspector holds.
fn profiled<'a>(sessions: &'a Sessions, app: &str) -> Result<(&'a Session, Profile, &'a NativeTree), Failure> {
    let asked = session(sessions, app)?;
    let profile = sessions.profile(asked.id).map_err(Failure::refused)?;
    let traced = sessions.clocked(asked.id).unwrap_or(asked);
    let tree = sessions
        .native_for(traced.id)
        .map(|inspector| &inspector.inspection.tree)
        .ok_or_else(|| Failure::refused("no native inspector: attach one first"))?;

    Ok((traced, profile, tree))
}

#[derive(Serialize)]
struct Seconds {
    budget_us: u64,
    seconds: Vec<SecondView>,
}

#[derive(Serialize)]
struct SecondView {
    t: i64,
    at: String,
    frames: usize,
    over: usize,
    worst_us: u64,
    worst_frame: i64,
}

/// Every second the last capture drew in, oldest first; a second nothing
/// was drawn in is not listed.
#[utoipa::path(
    get,
    path = "/apps/{app}/seconds",
    tag = "profile",
    operation_id = "get_seconds",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses(
        (status = 200, description = "Per second: frames, how many over 16.7 ms, the worst and which it was"),
        (status = 409, description = "No inspector, no frames captured on a clock, or no clock")
    )
)]
async fn seconds(State(state): State<crate::State>, Path(app): Path<String>) -> Result<Json<Seconds>, Failure> {
    state.read(|sessions| {
        let (_, profile, _) = profiled(sessions, &app)?;
        let seconds = profile
            .seconds()
            .into_iter()
            .map(|second| SecondView {
                at: profile.second_named(second.t),
                t: second.t,
                frames: second.frames,
                over: second.over,
                worst_us: second.worst_us,
                worst_frame: second.worst_frame,
            })
            .collect();

        Ok(Json(Seconds {
            budget_us: BUDGET_US,
            seconds,
        }))
    })
}

#[derive(Serialize)]
struct Frames {
    t: i64,
    at: String,
    frames: Vec<FrameLine>,
}

#[derive(Serialize)]
struct FrameLine {
    /// What names it in `/frames/{frame}`.
    frame: i64,
    at: String,
    took_us: u64,
    over: bool,
    measure_us: u64,
    arrange_us: u64,
    costliest: Option<PassView>,
    /// How many records the application made towards it.
    work: usize,
}

#[derive(Serialize)]
struct PassView {
    kind: String,
    element: u64,
    /// The element's type, while the inspector's tree still has it.
    element_kind: Option<String>,
    took_us: u64,
    at_us: u64,
}

impl PassView {
    fn of(pass: &Pass, tree: &NativeTree) -> Self {
        Self {
            kind: pass.kind.clone(),
            element: pass.element,
            element_kind: tree.get(pass.element).map(|element| element.kind.clone()),
            took_us: pass.took_us,
            at_us: pass.at_us,
        }
    }
}

/// The frames that began in second `t`, each in a line.
#[utoipa::path(
    get,
    path = "/apps/{app}/seconds/{t}/frames",
    tag = "profile",
    operation_id = "get_frames_of_second",
    params(
        ("app" = String, Path, description = "An application's id, or `latest`"),
        ("t" = i64, Path, description = "A second, as `/seconds` names it")
    ),
    responses(
        (status = 200, description = "The second's frames, oldest first"),
        (status = 409, description = "No inspector, no frames captured on a clock, or no clock")
    )
)]
async fn frames(
    State(state): State<crate::State>,
    Path((app, t)): Path<(String, i64)>,
) -> Result<Json<Frames>, Failure> {
    state.read(|sessions| {
        let (traced, profile, tree) = profiled(sessions, &app)?;
        let frames = profile
            .frames_in(t)
            .map(|frame| FrameLine {
                frame: frame.at_us,
                at: profile.when(frame.at_us),
                took_us: frame.took_us,
                over: frame.over(),
                measure_us: frame.measure_us,
                arrange_us: frame.arrange_us,
                costliest: frame.passes.first().map(|pass| PassView::of(pass, tree)),
                work: profile.work(frame, &traced.trace).len(),
            })
            .collect();

        Ok(Json(Frames {
            at: profile.second_named(t),
            t,
            frames,
        }))
    })
}

#[derive(Serialize)]
struct FrameView {
    frame: i64,
    at: String,
    took_us: u64,
    over: bool,
    measure_us: u64,
    arrange_us: u64,
    thread: u32,
    passes: Vec<PassView>,
    work: Vec<Record>,
}

#[derive(Serialize)]
struct Record {
    record: u64,
    parent: Option<u64>,
    at_us: u64,
    took_us: Option<u64>,
    says: String,
}

/// One frame whole: every layout pass, and what the application recorded
/// from where the frame before it ended to where it ends.
#[utoipa::path(
    get,
    path = "/apps/{app}/frames/{frame}",
    tag = "profile",
    operation_id = "get_frame",
    params(
        ("app" = String, Path, description = "An application's id, or `latest`"),
        ("frame" = i64, Path, description = "A frame, as `/seconds/{t}/frames` names it")
    ),
    responses(
        (status = 200, description = "The frame, its passes and the records towards it"),
        (status = 404, description = "No such frame in the last capture"),
        (status = 409, description = "No inspector, no frames captured on a clock, or no clock")
    )
)]
async fn frame(
    State(state): State<crate::State>,
    Path((app, at_us)): Path<(String, i64)>,
) -> Result<Json<FrameView>, Failure> {
    state.read(|sessions| {
        let (traced, profile, tree) = profiled(sessions, &app)?;
        let frame: &Frame = profile
            .frame(at_us)
            .ok_or_else(|| Failure::not_found(format!("no frame {at_us} in the last capture")))?;

        let work = profile
            .work(frame, &traced.trace)
            .into_iter()
            .map(|span| Record {
                record: span.id,
                parent: span.parent,
                at_us: span.at,
                took_us: span.took,
                says: words::text(&words::sentence(&span.point, &traced.timers)),
            })
            .collect();

        Ok(Json(FrameView {
            frame: frame.at_us,
            at: profile.when(frame.at_us),
            took_us: frame.took_us,
            over: frame.over(),
            measure_us: frame.measure_us,
            arrange_us: frame.arrange_us,
            thread: frame.thread,
            passes: frame.passes.iter().map(|pass| PassView::of(pass, tree)).collect(),
            work,
        }))
    })
}
