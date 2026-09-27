//! The puffin profiler, switched on from devtools and read over its own
//! connection.
//!
//! Frames do not travel through the devtools link: puffin has its own server
//! and its own format, and a frame of zones is far more data than a report.
//! Devtools are told where to connect and take it from there.

use std::sync::Mutex;

/// Where the server is asked to listen, first free port wins.
const PORTS: std::ops::Range<u16> = 8585..8600;

static RUNNING: Mutex<Option<Server>> = Mutex::new(None);

struct Server {
    at: String,
    /// Held: dropping it takes the listener and the frame sink away.
    #[cfg(feature = "profiling")]
    _server: puffin_http::Server,
}

/// Starts recording and listening; answers where. Starting twice answers
/// where it already listens.
pub fn start() -> anyhow::Result<String> {
    let mut running = RUNNING.lock().expect("the profiler lock");
    if let Some(server) = running.as_ref() {
        return Ok(server.at.clone());
    }

    let server = listen()?;
    let at = server.at.clone();
    *running = Some(server);

    guinea_core::devtools::profiling::record(true);
    Ok(at)
}

/// Stops recording and takes the server down.
pub fn stop() {
    guinea_core::devtools::profiling::record(false);
    let _ = RUNNING.lock().expect("the profiler lock").take();
}

#[cfg(feature = "profiling")]
fn listen() -> anyhow::Result<Server> {
    let mut last = None;
    for port in PORTS {
        let at = format!("127.0.0.1:{port}");
        match puffin_http::Server::new(&at) {
            Ok(server) => {
                return Ok(Server {
                    at,
                    _server: server,
                });
            }
            Err(error) => last = Some(error),
        }
    }

    Err(last.unwrap_or_else(|| anyhow::anyhow!("no port to listen on")))
}

#[cfg(not(feature = "profiling"))]
fn listen() -> anyhow::Result<Server> {
    anyhow::bail!("this application was built without the profiler")
}
