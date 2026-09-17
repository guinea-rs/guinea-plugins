//! The HTTP API: JSON of exactly what the model makes of each session.
//!
//! Every route is a `GET`, and wants `Authorization: Bearer <token>` with the
//! token from [`Access`]. `{app}` is an application's id, or `latest` for the
//! newest one still connected.
//!
//! - `/apps`: every application, newest first.
//! - `/apps/{app}/elements?closed=<id>,<id>&reveal=<id>`: the element tree.
//! - `/apps/{app}/element?id=<id>`: one element's details.
//! - `/apps/{app}/trace?hide=tick,send&text=kill&after=<record>&limit=200`:
//!   the last `limit` records the query lets through, after `after`.
//! - `/apps/{app}/trace/{record}?hide=tick`: one record, where it came from
//!   and what it set off.
//! - `/apps/{app}/streams`: what started the chains of records - actions,
//!   timers, loops - and how many each started.
//! - `/apps/{app}/chains?stream=timer/src/a.rs:4:9&text=sweep`: one stream's
//!   chains, those of one shape as one group; `stream=all` by default.
//! - `/apps/{app}/panels`: what plugins contributed.
//! - `/apps/{app}/graph`: actors, reducers and buses.
//! - `/apps/{app}/snapshot`: the last snapshot, as the application sent it.

use std::collections::{HashMap, HashSet};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use guinea_devtools_model::access::Access;
use guinea_devtools_model::chains::{self, Stream};
use guinea_devtools_model::elements::{self, Element};
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_model::trace::{self, Query};
use guinea_devtools_model::{graph, panels};
use serde::Serialize;
use tiny_http::{Header, Request, Response, Server};

use crate::Hub;

/// Where the API listens unless `GUINEA_DEVTOOLS_HTTP` says otherwise; any
/// free port when this one is taken.
const PREFERRED: &str = "127.0.0.1:47385";

/// How many trace records a request gets unless it asks for a number.
const LIMIT: usize = 200;

fn token() -> io::Result<String> {
    let mut bytes = [0u8; 24];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;

    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn bind() -> io::Result<(Server, SocketAddr)> {
    let wanted = std::env::var("GUINEA_DEVTOOLS_HTTP").unwrap_or_else(|_| PREFERRED.to_string());
    let server = Server::http(&wanted)
        .or_else(|_| Server::http("127.0.0.1:0"))
        .map_err(|error| io::Error::other(error.to_string()))?;

    let addr = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| io::Error::other("the API is not on an IP address"))?;

    Ok((server, addr))
}

/// Starts answering on its own thread, and writes down where and with what
/// token.
pub fn serve(hub: Arc<Hub>) -> io::Result<Access> {
    let (server, addr) = bind()?;
    let access = Access {
        url: format!("http://{addr}"),
        token: token()?,
    };
    access.write()?;

    let expected = format!("Bearer {}", access.token);
    std::thread::Builder::new()
        .name("devtools-http".into())
        .spawn(move || {
            for request in server.incoming_requests() {
                answer(&hub, &expected, request);
            }
        })?;

    Ok(access)
}

type Failure = (u16, String);

fn answer(hub: &Hub, expected: &str, request: Request) {
    let authorized = request
        .headers()
        .iter()
        .any(|header| header.field.equiv("Authorization") && header.value.as_str() == expected);

    let outcome = if !authorized {
        Err((401, "missing or wrong bearer token".to_string()))
    } else if *request.method() != tiny_http::Method::Get {
        Err((405, "only GET".to_string()))
    } else {
        route(hub, request.url())
    };

    let (status, body) = match outcome {
        Ok(body) => (200, body),
        Err((status, message)) => (status, serde_json::json!({ "error": message }).to_string()),
    };

    let json = Header::from_bytes("Content-Type", "application/json").expect("a valid header");
    let response = Response::from_string(body).with_status_code(status).with_header(json);

    if let Err(error) = request.respond(response) {
        tracing::debug!(%error, "could not answer");
    }
}

fn json(value: &impl Serialize) -> Result<String, Failure> {
    serde_json::to_string(value).map_err(|error| (500, error.to_string()))
}

fn session<'a>(sessions: &'a Sessions, app: &str) -> Result<&'a Session, Failure> {
    let found = if app == "latest" {
        sessions.newest().or_else(|| sessions.by_id.values().next_back())
    } else {
        app.parse().ok().and_then(|id| sessions.get(id))
    };

    found.ok_or_else(|| (404, format!("no application {app}")))
}

fn list(value: Option<&String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(',')
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn element(id: &str) -> Result<Element, Failure> {
    id.parse().map_err(|error| (400, error))
}

fn route(hub: &Hub, url: &str) -> Result<String, Failure> {
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();

    let sessions = hub.read();
    let trace_query = || Query {
        hidden: list(params.get("hide")),
        text: params.get("text").cloned().unwrap_or_default(),
        upto: None,
    };

    match parts.as_slice() {
        ["apps"] => json(&sessions.summaries()),
        ["apps", app, rest @ ..] => {
            let session = session(&sessions, app)?;

            match rest {
                ["snapshot"] => json(&session.snapshot),
                ["elements"] => {
                    let closed = list(params.get("closed"))
                        .iter()
                        .map(|id| element(id))
                        .collect::<Result<HashSet<_>, _>>()?;
                    let reveal = params.get("reveal").map(|id| element(id)).transpose()?;

                    json(&elements::lines(&session.snapshot, &closed, reveal.as_ref()))
                }

                ["element"] => {
                    let id = params.get("id").ok_or((400, "which element: ?id=".to_string()))?;
                    let details = elements::describe(session, &element(id)?)
                        .ok_or_else(|| (404, format!("no element {id}")))?;

                    json(&details)
                }

                ["trace"] => {
                    let query = trace_query();
                    let after: u64 = params.get("after").and_then(|after| after.parse().ok()).unwrap_or(0);
                    let limit: usize = params.get("limit").and_then(|limit| limit.parse().ok()).unwrap_or(LIMIT);

                    let shown: Vec<_> = trace::shown(session.reading(), &query)
                        .into_iter()
                        .filter(|span| span.id > after)
                        .collect();
                    let rows: Vec<_> = shown
                        .iter()
                        .skip(shown.len().saturating_sub(limit))
                        .map(|span| trace::row(session.reading(), span))
                        .collect();

                    json(&rows)
                }

                ["trace", record] => {
                    let id: u64 = record.parse().map_err(|_| (400, format!("not a record: {record}")))?;
                    let record = trace::record(session.reading(), id, &trace_query())
                        .ok_or_else(|| (404, format!("no record {id}")))?;

                    json(&record)
                }

                ["streams"] => json(&chains::streams(&session.chains, session.reading())),

                ["chains"] => {
                    let stream: Stream = params
                        .get("stream")
                        .map_or(Ok(Stream::All), |id| id.parse())
                        .map_err(|error| (400, error))?;
                    let text = params.get("text").cloned().unwrap_or_default();

                    json(&chains::view(&session.chains, session.reading(), &stream, &text))
                }

                ["panels"] => json(&panels::listed(&session.snapshot)),
                ["graph"] => json(&graph::build(&session.snapshot, &session.trace)),
                _ => Err((404, format!("no such route: {path}"))),
            }
        }
        _ => Err((404, format!("no such route: {path}"))),
    }
}
