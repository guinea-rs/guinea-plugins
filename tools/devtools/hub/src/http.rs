//! Serving the API: a port, a token, and the routes [`guinea_devtools_api`]
//! describes.
//!
//! Nothing about what the routes answer is here. This binds, checks the
//! bearer token, and hands the hub to the routes.

use std::io;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use guinea_devtools_api::Devtools;
use guinea_devtools_model::access::Access;

use crate::Hub;

/// Where the API listens unless `GUINEA_DEVTOOLS_HTTP` says otherwise; any
/// free port when this one is taken.
const PREFERRED: &str = "127.0.0.1:47385";

fn token() -> io::Result<String> {
    let mut bytes = [0u8; 24];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;

    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn bind() -> io::Result<(TcpListener, SocketAddr)> {
    let wanted = std::env::var("GUINEA_DEVTOOLS_HTTP").unwrap_or_else(|_| PREFERRED.to_string());
    let listener = TcpListener::bind(&wanted).or_else(|_| TcpListener::bind("127.0.0.1:0"))?;
    let addr = listener.local_addr()?;

    Ok((listener, addr))
}

/// What the routes may do with the hub.
fn devtools(hub: Arc<Hub>) -> Devtools {
    let reading = hub.clone();
    let sending = hub.clone();
    let attaching = hub;

    Devtools::new(
        move |job| job(&reading.read()),
        move |session, command| {
            sending
                .send(session, command)
                .map_err(|unsent| format!("{unsent:?}"))
        },
        move |app| attach(&attaching, app),
    )
}

#[cfg(windows)]
fn attach(hub: &Hub, app: u64) -> Result<(), String> {
    hub.attach_native(app)
}

#[cfg(not(windows))]
fn attach(_hub: &Hub, _app: u64) -> Result<(), String> {
    Err("the XAML tap is Windows only".to_string())
}

/// Starts answering on its own thread, and writes down where and with what
/// token.
pub fn serve(hub: Arc<Hub>) -> io::Result<Access> {
    let (listener, addr) = bind()?;
    listener.set_nonblocking(true)?;

    let access = Access {
        url: format!("http://{addr}"),
        token: token()?,
    };
    access.write()?;

    let (routes, document) = guinea_devtools_api::api(Arc::new(devtools(hub)));
    let expected: Arc<str> = format!("Bearer {}", access.token).into();

    let app = routes
        .route(
            "/openapi.json",
            axum::routing::get(move || {
                let document = document.clone();

                async move { axum::Json(document) }
            }),
        )
        .layer(axum::middleware::from_fn(move |request, next| {
            authorized(expected.clone(), request, next)
        }));

    std::thread::Builder::new()
        .name("devtools-http".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(runtime) => runtime,
                Err(error) => return tracing::error!(%error, "the API has no runtime"),
            };

            runtime.block_on(async move {
                let listener = match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => listener,
                    Err(error) => return tracing::error!(%error, "the API cannot listen"),
                };

                if let Err(error) = axum::serve(listener, app).await {
                    tracing::error!(%error, "the API stopped");
                }
            });
        })?;

    Ok(access)
}

/// Lets a request through when it carries the token, and says so when it does
/// not.
async fn authorized(expected: Arc<str>, request: Request, next: Next) -> Response {
    let carried = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    if carried != Some(&*expected) {
        let said = serde_json::json!({ "error": "missing or wrong bearer token" });

        return (StatusCode::UNAUTHORIZED, axum::Json(said)).into_response();
    }

    next.run(request).await
}
