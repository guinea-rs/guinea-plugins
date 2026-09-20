//! The devtools HTTP API: what it answers, and the document that says so.
//!
//! Nothing here holds state or talks to applications. What the routes need -
//! the sessions to read, a command to send, a tap to load - arrives as
//! [`Devtools`], which the hub fills in. So this crate can be asked what the
//! API looks like without starting one.
//!
//! The OpenAPI document is collected from the routes themselves
//! ([`document`]), which is why there is no second list of them anywhere:
//! `--help` renders that document, and so does the MCP server.

mod error;
mod routes;

pub mod help;
pub mod tools;

use std::sync::Arc;

use guinea_devtools_model::sessions::Sessions;
use guinea_devtools_protocol::Command;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub use error::Failure;

/// Reads every session, for as long as the job it is handed takes.
type Reading = Box<dyn Fn(&mut dyn FnMut(&Sessions)) + Send + Sync>;
/// Asks one session for something, and says why it could not.
type Asking = Box<dyn Fn(u64, Command) -> Result<(), String> + Send + Sync>;
/// Loads a native inspector into one application.
type Attaching = Box<dyn Fn(u64) -> Result<(), String> + Send + Sync>;

/// What the routes are allowed to do, handed in rather than reached for.
pub struct Devtools {
    sessions: Reading,
    send: Asking,
    attach: Attaching,
}

impl Devtools {
    /// `sessions` reads them under whatever lock the hub keeps them behind,
    /// `send` queues a command for one session, `attach` loads a native
    /// inspector into one application.
    pub fn new(
        sessions: impl Fn(&mut dyn FnMut(&Sessions)) + Send + Sync + 'static,
        send: impl Fn(u64, Command) -> Result<(), String> + Send + Sync + 'static,
        attach: impl Fn(u64) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            sessions: Box::new(sessions),
            send: Box::new(send),
            attach: Box::new(attach),
        }
    }

    /// What `job` makes of the sessions.
    pub fn read<R>(&self, job: impl FnOnce(&Sessions) -> R) -> R {
        let mut job = Some(job);
        let mut answer = None;

        (self.sessions)(&mut |sessions| {
            let job = job.take().expect("the sessions are read once");
            answer = Some(job(sessions));
        });

        answer.expect("reading the sessions runs the job")
    }

    pub fn send(&self, session: u64, command: Command) -> Result<(), String> {
        (self.send)(session, command)
    }

    pub fn attach(&self, app: u64) -> Result<(), String> {
        (self.attach)(app)
    }
}

/// The state every route is handed.
pub type State = Arc<Devtools>;

/// The API's title, and what its routes have in common.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "guinea devtools",
        description = "JSON of what devtools make of each connected application. \
Every route wants `Authorization: Bearer <token>`; the address and the token are \
printed at startup and kept in `devtools.http.json` beside the key. Where a route \
takes `{app}`, that is an application's id or `latest` for the newest one still \
connected."
    ),
    tags(
        (name = "apps", description = "What connected, and what it is made of"),
        (name = "trace", description = "What the application did, record by record"),
        (name = "native", description = "The backend's own elements, through an inspector")
    )
)]
struct Api;

/// The routes, and the document that describes exactly them.
pub fn api(state: State) -> (axum::Router, utoipa::openapi::OpenApi) {
    let (router, document) = OpenApiRouter::with_openapi(Api::openapi())
        .merge(routes::router())
        .with_state(state)
        .split_for_parts();

    (router, document)
}

/// The document alone, for a process that describes the API without serving
/// it.
pub fn document() -> utoipa::openapi::OpenApi {
    OpenApiRouter::with_openapi(Api::openapi())
        .merge(routes::router())
        .split_for_parts()
        .1
}
