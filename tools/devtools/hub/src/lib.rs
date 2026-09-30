//! Where applications report to devtools, kept in one place that the window
//! and the HTTP API both read.

mod http;
#[cfg(windows)]
mod integrity;
mod listen;

use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex, OnceLock, RwLock, RwLockReadGuard};

use guinea_devtools_model::sessions::{Incoming, Sessions};
use guinea_devtools_protocol::Command;

pub use http::serve;

type Watcher = Box<dyn Fn() + Send + Sync>;

/// What every application reported, as one [`Sessions`].
#[derive(Default)]
pub struct Hub {
    sessions: RwLock<Sessions>,
    watchers: Mutex<Vec<Watcher>>,
    commands: OnceLock<listen::Commands>,
}

/// Why a command was not sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unsent {
    NoSuchSession,
    Disconnected,
    /// The session never said it can do what the command needs.
    NotOffered,
    /// The listener is gone.
    Closed,
}

/// How many reports are applied under one write before readers get a turn.
const BATCH: usize = 512;

impl Hub {
    /// Starts listening for applications.
    pub fn start() -> Arc<Hub> {
        let hub = Arc::new(Hub::default());
        let (out, inbox) = channel();
        let _ = hub.commands.set(listen::spawn(out));

        let applying = hub.clone();
        std::thread::Builder::new()
            .name("devtools-apply".into())
            .spawn(move || applying.apply_all(inbox))
            .expect("spawning the applying thread");

        hub
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Sessions> {
        self.sessions.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sends `command` to session `id`, if it listed what the command needs.
    ///
    /// Queued, not waited on: whatever the session answers arrives as a
    /// report like any other.
    pub fn send(&self, id: u64, command: Command) -> Result<(), Unsent> {
        self.send_in(&self.read(), id, command)
    }

    /// [`Hub::send`] for a caller that already holds [`Hub::read`]: reading
    /// again on the same thread deadlocks as soon as a report waits to be
    /// applied.
    pub fn send_in(&self, sessions: &Sessions, id: u64, command: Command) -> Result<(), Unsent> {
        let session = sessions.get(id).ok_or(Unsent::NoSuchSession)?;

        if !session.connected {
            return Err(Unsent::Disconnected);
        }
        if !session.info.can(command.needs()) {
            return Err(Unsent::NotOffered);
        }

        let commands = self.commands.get().ok_or(Unsent::Closed)?;
        commands.unbounded_send((id, command)).map_err(|_| Unsent::Closed)
    }

    /// Loads the XAML tap into session `id`'s process, unless one is already
    /// there. The tap connects on its own and shows up as a session with the
    /// same pid.
    ///
    /// What is loaded is a copy: a process keeps its tap until it exits, and
    /// the one next to devtools has to stay free to be rebuilt.
    #[cfg(windows)]
    pub fn attach_native(&self, id: u64) -> Result<(), String> {
        attach_native_in(&self.read(), id)
    }

    /// Calls `watcher` on the applying thread after every batch of reports.
    pub fn on_change(&self, watcher: impl Fn() + Send + Sync + 'static) {
        self.watchers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Box::new(watcher));
    }

    fn apply_all(&self, inbox: Receiver<Incoming>) {
        while let Ok(first) = inbox.recv() {
            {
                let mut sessions = self.sessions.write().unwrap_or_else(|poisoned| poisoned.into_inner());
                sessions.apply(first);

                for next in inbox.try_iter().take(BATCH) {
                    sessions.apply(next);
                }
            }

            for watcher in self.watchers.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).iter() {
                watcher();
            }
        }
    }
}

/// [`Hub::attach_native`] for a caller that already holds [`Hub::read`].
#[cfg(windows)]
pub fn attach_native_in(sessions: &Sessions, id: u64) -> Result<(), String> {
    let session = sessions.get(id).ok_or("no such session")?;
    if sessions.native_for(id).is_some() {
        return Ok(());
    }
    let pid = session.info.pid;

    match (integrity::own(), integrity::of(pid)) {
        (Some(ours), Some(theirs)) if ours <= theirs => {}
        (Some(_), Some(_)) => {
            return Err(
                "devtools run elevated and the application does not: attaching would load its \
                 WinUI runtime into devtools. Start devtools without elevation"
                    .into(),
            );
        }
        _ => return Err(format!("cannot tell how far process {pid} is trusted")),
    }

    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let beside = exe.with_file_name(guinea_xaml_tap::DLL);
    let in_deps = exe.with_file_name("deps").join(guinea_xaml_tap::DLL);
    let Some(built) = [&beside, &in_deps].into_iter().find(|path| path.exists()) else {
        return Err(format!(
            "no tap at {} - build it with `cargo build --workspace` in tools/devtools and keep \
             {} beside the executable",
            beside.display(),
            guinea_xaml_tap::DLL
        ));
    };

    let temp = std::env::temp_dir();
    let copies = |entry: &std::fs::DirEntry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name.starts_with(TAP_COPY) && name.ends_with(".dll")
    };
    for stale in std::fs::read_dir(&temp).into_iter().flatten().flatten().filter(copies) {
        let _ = std::fs::remove_file(stale.path());
    }

    let mut unique = [0u8; 8];
    getrandom::fill(&mut unique).map_err(|error| format!("no name for the tap's copy: {error}"))?;
    let loaded = temp.join(format!("{TAP_COPY}{pid}-{:016x}.dll", u64::from_le_bytes(unique)));
    std::fs::copy(built, &loaded)
        .map_err(|error| format!("copying the tap to {}: {error}", loaded.display()))?;

    guinea_xaml_tap::inject::inject(pid, &loaded)
}

/// What every copy of the tap in the temporary directory is named from. A
/// copy still loaded somewhere cannot be removed, and is left alone.
#[cfg(windows)]
const TAP_COPY: &str = "guinea-xaml-tap-";
