//! What the application did: records, and the chains they form.

use axum::Json;
use axum::extract::{Path, Query, State};
use guinea_devtools_model::chains::{self, Stream, StreamLine, StreamView};
use guinea_devtools_model::trace::{self, About, Class, Record, Row};
use guinea_devtools_model::words::Level;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::{list, session};
use crate::Failure;

/// How many records a request gets unless it asks for a number.
const LIMIT: usize = 200;

pub fn router() -> OpenApiRouter<crate::State> {
    OpenApiRouter::new()
        .routes(routes!(records))
        .routes(routes!(record))
        .routes(routes!(about))
        .routes(routes!(streams))
        .routes(routes!(chains))
}

#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Filter {
    /// Only these: `own` for what the application's code wrote, a level -
    /// `error`, `warn`, `info`, `debug`, `trace` - for its own at that level,
    /// `slow` for the framework's work of a millisecond or more. Every record
    /// when left out.
    class: Option<String>,
    /// Kinds left out, comma-separated: `action send handle spawn settled
    /// cancelled publish deliver push navigate store render log tick note`.
    #[serde(default)]
    hide: String,
    /// Only records whose line says this.
    #[serde(default)]
    text: String,
    /// Only records newer than this one, for polling.
    #[serde(default)]
    after: u64,
    /// How many of the newest records that are left.
    limit: Option<usize>,
}

impl Filter {
    fn query(&self) -> Result<trace::Query, String> {
        let class = match self.class.as_deref() {
            None => Class::Every,
            Some("own") => Class::Own,
            Some("slow") => Class::Slow,
            Some(name) => Level::parse(name)
                .map(Class::Level)
                .ok_or_else(|| format!("not a class of records: {name}"))?,
        };

        Ok(trace::Query {
            class,
            hidden: list(&self.hide),
            text: self.text.clone(),
            upto: None,
        })
    }
}

/// The last records the filter lets through, oldest first.
#[utoipa::path(
    get,
    path = "/apps/{app}/trace",
    tag = "trace",
    operation_id = "get_trace",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Filter),
    responses((status = 200, description = "One row per record, as words with tones"))
)]
async fn records(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<Row>>, Failure> {
    let query = filter.query().map_err(Failure::bad_request)?;
    let limit = filter.limit.unwrap_or(LIMIT);

    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let shown: Vec<_> = trace::shown(session.reading(), &query)
            .into_iter()
            .filter(|span| span.id > filter.after)
            .collect();

        let rows = shown
            .iter()
            .skip(shown.len().saturating_sub(limit))
            .map(|span| trace::row(session.reading(), span))
            .collect();

        Ok(Json(rows))
    })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Between {
    /// Kinds left out of what happened in between.
    #[serde(default)]
    hide: String,
}

/// One record: what caused it, and what it set off.
#[utoipa::path(
    get,
    path = "/apps/{app}/trace/{record}",
    tag = "trace",
    operation_id = "get_record",
    params(
        ("app" = String, Path, description = "An application's id, or `latest`"),
        ("record" = u64, Path, description = "The record's id"),
        Between
    ),
    responses(
        (status = 200, description = "The record, its causes and its consequences"),
        (status = 404, description = "No record with that id is kept")
    )
)]
async fn record(
    State(state): State<crate::State>,
    Path((app, id)): Path<(String, u64)>,
    Query(between): Query<Between>,
) -> Result<Json<Record>, Failure> {
    let query = trace::Query {
        hidden: list(&between.hide),
        ..trace::Query::default()
    };

    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let record = trace::record(session.reading(), id, &query)
            .ok_or_else(|| Failure::not_found(format!("no record {id}")))?;

        Ok(Json(record))
    })
}

/// One record as the trace page's side panel reads it: what it says, where
/// it was written, what the framework was doing and what ran inside it.
#[utoipa::path(
    get,
    path = "/apps/{app}/trace/{record}/about",
    tag = "trace",
    operation_id = "get_about",
    params(
        ("app" = String, Path, description = "An application's id, or `latest`"),
        ("record" = u64, Path, description = "The record's id")
    ),
    responses(
        (status = 200, description = "The record, what it ran in and what ran inside it"),
        (status = 404, description = "No record with that id is kept")
    )
)]
async fn about(
    State(state): State<crate::State>,
    Path((app, id)): Path<(String, u64)>,
) -> Result<Json<About>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;
        let about = trace::about(session.reading(), id)
            .ok_or_else(|| Failure::not_found(format!("no record {id}")))?;

        Ok(Json(about))
    })
}

/// What started the chains, and how many each started.
#[utoipa::path(
    get,
    path = "/apps/{app}/streams",
    tag = "trace",
    operation_id = "get_streams",
    params(("app" = String, Path, description = "An application's id, or `latest`")),
    responses((status = 200, description = "Each stream, named and counted"))
)]
async fn streams(
    State(state): State<crate::State>,
    Path(app): Path<String>,
) -> Result<Json<Vec<StreamLine>>, Failure> {
    state.read(|sessions| {
        let session = session(sessions, &app)?;

        Ok(Json(chains::streams(&session.chains, session.reading())))
    })
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Which {
    /// A stream as `/apps/{app}/streams` names it, `timer/src/a.rs:4:9`;
    /// every chain when left out.
    stream: Option<String>,
    /// Only chains that say this.
    #[serde(default)]
    text: String,
}

/// One stream's chains, those of one shape grouped together.
#[utoipa::path(
    get,
    path = "/apps/{app}/chains",
    tag = "trace",
    operation_id = "get_chains",
    params(("app" = String, Path, description = "An application's id, or `latest`"), Which),
    responses(
        (status = 200, description = "The chains, grouped by shape"),
        (status = 400, description = "That is not a stream")
    )
)]
async fn chains(
    State(state): State<crate::State>,
    Path(app): Path<String>,
    Query(which): Query<Which>,
) -> Result<Json<StreamView>, Failure> {
    let stream: Stream = which
        .stream
        .map_or(Ok(Stream::All), |id| id.parse())
        .map_err(Failure::bad_request)?;

    state.read(|sessions| {
        let session = session(sessions, &app)?;

        Ok(Json(chains::view(
            &session.chains,
            session.reading(),
            &stream,
            &which.text,
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_class_is_asked_for_by_name() {
        let class = |name: Option<&str>| {
            Filter {
                class: name.map(str::to_string),
                ..Filter::default()
            }
            .query()
            .map(|query| query.class)
        };

        assert_eq!(class(None), Ok(Class::Every));
        assert_eq!(class(Some("own")), Ok(Class::Own));
        assert_eq!(class(Some("warn")), Ok(Class::Level(Level::Warn)));
        assert_eq!(class(Some("slow")), Ok(Class::Slow));
        assert!(class(Some("loud")).is_err());
    }
}
