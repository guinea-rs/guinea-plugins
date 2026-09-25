//! Driving an application: an action to its features, an event on its bus,
//! a click or keystrokes on its elements.
//!
//! Unlike the rest of the API these wait. The application answers each one
//! under a request id, and the route holds on until it has; an action or an
//! event is then followed through the trace until what it set off has gone
//! quiet, so the answer says what it did.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Path, Query, State};
use guinea_devtools_model::protocol::native::{Bounds, Input, Target};
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_model::words;
use guinea_devtools_protocol::{Answer, Capability, Command};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::session;
use crate::Failure;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .routes(routes!(remote))
        .routes(routes!(act))
        .routes(routes!(publish))
        .routes(routes!(find))
        .routes(routes!(click))
        .routes(routes!(type_text))
}

/// How long the application has to answer.
const ANSWER: Duration = Duration::from_secs(10);
/// How often an answer, or the trace, is looked at again.
const POLL: Duration = Duration::from_millis(50);
/// How long what an action set off has to stay the same to count as done:
/// a little over two of the plugin's trace batches, since a hop to the UI
/// thread shows up only once it has run.
const QUIET: Duration = Duration::from_millis(600);
/// How long an action is followed when the caller does not say.
const FOLLOWED: Duration = Duration::from_secs(5);

static NEXT: AtomicU64 = AtomicU64::new(1);

/// What a tool may do to one application.
#[derive(Serialize)]
struct Remote {
    /// Actions `act` can send, by name.
    actions: Vec<String>,
    /// Events `publish` can put on the global bus, by name.
    events: Vec<String>,
    /// Whether the `native/…` routes can find, click and type into its
    /// elements - which needs a native inspector.
    input: bool,
}

/// What a tool may send this application, and whether its elements can be
/// clicked.
#[utoipa::path(
    get,
    path = "/apps/{app}/remote",
    tag = "drive",
    operation_id = "get_remote",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses((status = 200, description = "The actions, the events, and whether input reaches its elements"))
)]
async fn remote(State(state): State<crate::State>, Path(app): Path<String>) -> Result<Json<Remote>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let input = sessions
            .native_for(session.id)
            .is_some_and(|inspector| inspector.info.can(Capability::NativeInput));

        Ok(Json(Remote {
            actions: session.info.actions.clone(),
            events: session.info.events.clone(),
            input,
        }))
    })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Acting {
    /// The action, by the name `remote` lists it under.
    action: String,
    /// The action as JSON: `"guinea"` for `Rename(String)`, `{"pid": 42}`
    /// for a struct. Left out, `null` - which is what a unit struct is.
    payload: Option<String>,
    /// The window whose page answers it; the newest when left out.
    root: Option<u64>,
    /// How many milliseconds to follow what it sets off; `0` answers as soon
    /// as it is sent. 5000 when left out.
    wait: Option<u64>,
}

/// What an action or an event set off.
#[derive(Serialize)]
struct Acted {
    /// Its record in the trace; what it set off is under it.
    cause: u64,
    /// Whether it went quiet within the wait: nothing it started is still
    /// running, and nothing new has come of it for a moment.
    settled: bool,
    /// The record and everything under it, one line each, indented by how
    /// far down it is.
    chain: Vec<String>,
}

/// Sends an action to the scope on the open page that answers it, and
/// follows what it sets off.
#[utoipa::path(
    post,
    path = "/apps/{app}/act",
    tag = "drive",
    operation_id = "send_action",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Acting),
    responses(
        (status = 200, description = "Sent; what it set off, and whether that went quiet"),
        (status = 409, description = "Not sent: no such action, bad JSON, or nothing answers it"),
        (status = 504, description = "The application did not answer")
    )
)]
async fn act(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(acting): Query<Acting>,
) -> Result<Json<Acted>, Failure> {
    let session = peer(&state, &app, Capability::Act, false)?;
    let wait = acting.wait.map_or(FOLLOWED, Duration::from_millis);

    let answer = ask(&state, session, |request| Command::Act {
        request,
        root: acting.root,
        action: acting.action,
        payload: acting.payload.unwrap_or_else(|| "null".to_string()),
    })
    .await?;

    followed(&state, session, answer, wait).await
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Publishing {
    /// The event, by the name `remote` lists it under.
    event: String,
    /// The event as JSON. Left out, `null`.
    payload: Option<String>,
    /// As for `send_action`.
    wait: Option<u64>,
}

/// Publishes an event on the application's global bus, as if it had arrived
/// on the UI thread, and follows what it sets off.
#[utoipa::path(
    post,
    path = "/apps/{app}/publish",
    tag = "drive",
    operation_id = "publish_event",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Publishing),
    responses(
        (status = 200, description = "Published; what it set off, and whether that went quiet"),
        (status = 409, description = "Not published: no such event, or bad JSON"),
        (status = 504, description = "The application did not answer")
    )
)]
async fn publish(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(publishing): Query<Publishing>,
) -> Result<Json<Acted>, Failure> {
    let session = peer(&state, &app, Capability::Act, false)?;
    let wait = publishing.wait.map_or(FOLLOWED, Duration::from_millis);

    let answer = ask(&state, session, |request| Command::Publish {
        request,
        event: publishing.event,
        payload: publishing.payload.unwrap_or_else(|| "null".to_string()),
    })
    .await?;

    followed(&state, session, answer, wait).await
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Aimed {
    /// The mark the element carries - its `AutomationId`.
    mark: String,
    /// Which of the elements that carry it, in tree order. 0 when left out.
    nth: Option<usize>,
    /// Only elements under the first one marked this - the row a button is
    /// in.
    within: Option<String>,
    /// `pointer` - the real mouse and keyboard, which moves the cursor and
    /// brings the window forward - or `automation`, the control's own
    /// pattern, for controls that have one. `pointer` when left out.
    input: Option<String>,
}

impl Aimed {
    fn target(&self) -> Target {
        Target {
            mark: self.mark.clone(),
            nth: self.nth.unwrap_or(0),
            within: self.within.clone(),
        }
    }

    fn input(&self) -> Result<Input, Failure> {
        match self.input.as_deref() {
            None | Some("pointer") => Ok(Input::Pointer),
            Some("automation") => Ok(Input::Automation),
            Some(other) => Err(Failure::bad_request(format!(
                "input is `pointer` or `automation`, not {other:?}"
            ))),
        }
    }
}

#[derive(Serialize)]
struct Found {
    /// The element, as `get_native_tree` names it.
    element: u64,
    /// Where it is on the screen, in physical pixels.
    bounds: Option<Bounds>,
}

/// Finds an element by the mark it carries.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/find",
    tag = "drive",
    operation_id = "find_element",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Aimed),
    responses(
        (status = 200, description = "The element and where it is"),
        (status = 409, description = "Nothing carries that mark, or there is no inspector"),
        (status = 504, description = "The inspector did not answer")
    )
)]
async fn find(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(aimed): Query<Aimed>,
) -> Result<Json<Found>, Failure> {
    let inspector = peer(&state, &app, Capability::NativeInput, true)?;
    let target = aimed.target();

    match ask(&state, inspector, |request| Command::NativeFind { request, target }).await? {
        Answer::Found { element, bounds } => Ok(Json(Found { element, bounds })),
        other => Err(unexpected(other)),
    }
}

#[derive(Serialize)]
struct Done {
    done: bool,
}

/// Clicks an element by the mark it carries.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/click",
    tag = "drive",
    operation_id = "click_element",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Aimed),
    responses(
        (status = 200, description = "Clicked; what it changed shows up in the next read"),
        (status = 409, description = "Not clicked: no such element, covered, or no pattern to click with"),
        (status = 504, description = "The inspector did not answer")
    )
)]
async fn click(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(aimed): Query<Aimed>,
) -> Result<Json<Done>, Failure> {
    let inspector = peer(&state, &app, Capability::NativeInput, true)?;
    let (target, input) = (aimed.target(), aimed.input()?);

    match ask(&state, inspector, |request| Command::NativeClick {
        request,
        target,
        input,
    })
    .await?
    {
        Answer::Done => Ok(Json(Done { done: true })),
        other => Err(unexpected(other)),
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Typing {
    /// What to type.
    text: String,
}

/// Types into an element by the mark it carries: clicked first and then
/// keystrokes with the pointer, the value set outright with automation.
#[utoipa::path(
    post,
    path = "/apps/{app}/native/type",
    tag = "drive",
    operation_id = "type_into_element",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Aimed, Typing),
    responses(
        (status = 200, description = "Typed; what it changed shows up in the next read"),
        (status = 409, description = "Not typed: no such element, covered, or it takes no value"),
        (status = 504, description = "The inspector did not answer")
    )
)]
async fn type_text(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(aimed): Query<Aimed>,
    Query(typing): Query<Typing>,
) -> Result<Json<Done>, Failure> {
    let inspector = peer(&state, &app, Capability::NativeInput, true)?;
    let (target, input) = (aimed.target(), aimed.input()?);

    match ask(&state, inspector, |request| Command::NativeType {
        request,
        target,
        text: typing.text,
        input,
    })
    .await?
    {
        Answer::Done => Ok(Json(Done { done: true })),
        other => Err(unexpected(other)),
    }
}

/// The session that takes a command needing `capability`: the application's
/// own, or with `native` its inspector.
fn peer(state: &crate::State, app: &str, capability: Capability, native: bool) -> Result<u64, Failure> {
    state.read(|sessions: &Sessions| {
        let session = session(sessions, app)?;
        let peer = if native {
            sessions.native_for(session.id).ok_or_else(|| {
                Failure::refused(format!(
                    "{} has no native inspector yet - POST /apps/{app}/native/attach",
                    session.name()
                ))
            })?
        } else {
            session
        };

        if peer.info.can(capability) {
            Ok(peer.id)
        } else {
            Err(Failure::refused(format!(
                "{} cannot be asked for that - it does not list {capability:?}",
                peer.name()
            )))
        }
    })
}

/// Sends the command `made` makes of a fresh request id, and waits for the
/// answer to it.
async fn ask(state: &crate::State, session: u64, made: impl FnOnce(u64) -> Command) -> Result<Answer, Failure> {
    let request = NEXT.fetch_add(1, Ordering::Relaxed);
    state.send(session, made(request)).map_err(Failure::refused)?;

    let asked = Instant::now();
    loop {
        let answer = state.read(|sessions| {
            sessions
                .get(session)
                .and_then(|session| session.answers.get(&request).cloned())
        });

        match answer {
            Some(Answer::Refused { reason }) => return Err(Failure::refused(reason)),
            Some(answer) => return Ok(answer),
            None if asked.elapsed() > ANSWER => {
                return Err(Failure::timed_out(format!("no answer within {ANSWER:?}")));
            }
            None => tokio::time::sleep(POLL).await,
        }
    }
}

/// Follows an action's cause through the trace until it goes quiet or
/// `wait` runs out.
async fn followed(state: &crate::State, session: u64, answer: Answer, wait: Duration) -> Result<Json<Acted>, Failure> {
    let Answer::Acted { cause } = answer else {
        return Err(unexpected(answer));
    };

    let started = Instant::now();
    let mut seen = None;
    let mut still_since = Instant::now();

    loop {
        let (arrived, under, unfinished, chain) = state.read(|sessions| {
            sessions.get(session).map_or((false, 0, 0, Vec::new()), |session| {
                (
                    session.trace.get(cause).is_some(),
                    session.trace.under(cause).len(),
                    session.trace.unfinished(cause),
                    lines(session, cause),
                )
            })
        });

        if seen != Some(under) {
            seen = Some(under);
            still_since = Instant::now();
        }

        let settled = arrived && unfinished == 0 && still_since.elapsed() >= QUIET;
        if settled || started.elapsed() >= wait {
            return Ok(Json(Acted { cause, settled, chain }));
        }

        tokio::time::sleep(POLL).await;
    }
}

/// `cause` and everything under it, a line each.
fn lines(session: &Session, cause: u64) -> Vec<String> {
    let said = |point| words::text(&words::sentence(point, &session.timers));

    session
        .trace
        .get(cause)
        .map(|root| said(&root.point))
        .into_iter()
        .chain(
            session
                .trace
                .under(cause)
                .into_iter()
                .map(|(depth, span)| format!("{}{}", "  ".repeat(depth), said(&span.point))),
        )
        .collect()
}

fn unexpected(answer: Answer) -> Failure {
    Failure::refused(format!("the application answered something else: {answer:?}"))
}
