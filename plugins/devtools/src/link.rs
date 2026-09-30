//! The connection, on a thread and a runtime of its own.
//!
//! Reports go through a short queue. When devtools are not there, or not
//! keeping up, they are dropped: a snapshot is replaced by the next one anyway,
//! and the application must never wait on its own debugger.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};
use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{Answer, AppInfo, Command, Report, key, wire};
use ogurpchik::rpc::connect_session;

const QUEUE: usize = 64;
const RETRY: Duration = Duration::from_secs(1);

pub struct Outbox(Sender<Report>);

impl Drop for Outbox {
    fn drop(&mut self) {
        self.0.close_channel();
    }
}

impl Outbox {
    pub fn send(&mut self, report: Report) {
        if let Err(error) = self.0.try_send(report)
            && error.is_disconnected()
        {
            tracing::trace!("the devtools link is gone");
        }
    }
}

/// Starts the link. `connected` is true while devtools are there, so that
/// nothing is collected for nobody. With `launch`, the first attempt that
/// finds nobody starts devtools.
pub fn spawn(info: Arc<Mutex<AppInfo>>, connected: Arc<AtomicBool>, launch: bool) -> Outbox {
    let (sender, receiver) = channel(QUEUE);
    let answers = sender.clone();
    let spawned = std::thread::Builder::new()
        .name("guinea-devtools".into())
        .spawn(move || match compio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(run(info, connected, launch, receiver, answers)),
            Err(error) => tracing::warn!(%error, "the devtools link has no runtime"),
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "the devtools link did not start");
    }
    Outbox(sender)
}

/// Takes the commands devtools send, and answers on the queue the reports go
/// out on.
struct Inbound(Mutex<Sender<Report>>);

impl Inbound {
    fn answer(&self, report: Report) {
        if let Ok(mut reports) = self.0.lock() {
            let _ = reports.try_send(report);
        }
    }

    /// Runs `act` on the UI thread, where scopes and the global bus live, and
    /// answers `request` with the trace point it made or why it could not.
    fn on_ui(&self, request: u64, act: impl FnOnce() -> Result<u64, String> + Send + 'static) {
        let Ok(reports) = self.0.lock().map(|reports| reports.clone()) else {
            return;
        };

        guinea_core::actor::invoke_on_ui(move || {
            let answer = match act() {
                Ok(cause) => Answer::Acted { cause },
                Err(reason) => Answer::Refused { reason },
            };

            let mut reports = reports;
            let _ = reports.try_send(Report::Answered { request, answer });
        });
    }
}

impl peer::Server for Inbound {
    async fn send(
        self: capnp::capability::Rc<Self>,
        params: peer::SendParams,
        _results: peer::SendResults,
    ) -> Result<(), capnp::Error> {
        let command: Command = wire::received(&params)?;

        match command {
            Command::Profiler { on: true } => match crate::profiler::start() {
                Ok(at) => self.answer(Report::Profiler { at: Some(at) }),
                Err(error) => self.answer(Report::Refused {
                    command: "Profiler".to_string(),
                    reason: error.to_string(),
                }),
            },
            Command::Profiler { on: false } => {
                crate::profiler::stop();
                self.answer(Report::Profiler { at: None });
            }
            Command::Act {
                request,
                root,
                action,
                payload,
            } => self.on_ui(request, move || {
                guinea::devtools::act(root, &action, &payload)
            }),
            Command::Publish {
                request,
                event,
                payload,
            } => self.on_ui(request, move || {
                guinea_core::remote::publish(&event, &payload)
            }),
            other => {
                tracing::debug!(
                    ?other,
                    "devtools asked for something this link does not offer"
                );
            }
        }

        Ok(())
    }
}

async fn run(
    info: Arc<Mutex<AppInfo>>,
    connected: Arc<AtomicBool>,
    mut launch: bool,
    mut reports: Receiver<Report>,
    answers: Sender<Report>,
) {
    loop {
        let endpoint = match guinea_devtools_protocol::endpoint() {
            Ok(endpoint) => endpoint,
            Err(error) => {
                tracing::warn!(%error, "nowhere to reach devtools at");
                if !idle(&mut reports).await {
                    return;
                }
                continue;
            }
        };
        let session = match key::read() {
            Ok(secret) => {
                let inbound = Inbound(Mutex::new(answers.clone()));
                connect_session::<peer::Client, _>(
                    &endpoint,
                    &key::handshake(secret),
                    guinea_devtools_protocol::PROTOCOL,
                    inbound,
                )
                .await
                .ok()
            }
            Err(_) => None,
        };
        let Some(session) = session else {
            if std::mem::take(&mut launch) {
                crate::launch::start();
            }
            if !idle(&mut reports).await {
                return;
            }
            continue;
        };
        tracing::debug!(%endpoint, "connected to devtools");
        connected.store(true, Ordering::Relaxed);

        let remote = session.remote();
        let mut announced = AppInfo::default();
        loop {
            let current = info
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();
            if current != announced {
                if wire::send(remote, &Report::Hello(current.clone()))
                    .await
                    .is_err()
                {
                    break;
                }
                announced = current;
            }

            let Some(report) = reports.next().await else {
                crate::profiler::stop();
                return;
            };
            if wire::send(remote, &report).await.is_err() {
                tracing::debug!(%endpoint, "devtools went away");
                break;
            }
        }

        crate::profiler::stop();
        connected.store(false, Ordering::Relaxed);
    }
}

/// Throws away what arrives while there is no connection. `false` once the
/// application has gone.
async fn idle(reports: &mut Receiver<Report>) -> bool {
    let drained =
        compio::time::timeout(RETRY, async { while reports.next().await.is_some() {} }).await;
    drained.is_err()
}
