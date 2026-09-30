//! Keeps one copy of an application running per user.
//!
//! ```no_run
//! # use guinea_plugin_single_instance::SingleInstancePlugin;
//! guinea::app::GuineaApp::new()
//!     .meta(guinea::app::AppMeta::new("app", "dev.example.app", "0.1.0", "example"))
//!     .plugin(SingleInstancePlugin::new())
//!     # ;
//! ```
//!
//! The first copy holds a lock on a file named after the application's
//! identifier, in the user's own profile, for as long as it runs. A copy that
//! finds the lock taken leaves before it opens a window. The operating system
//! drops the lock when the holder exits, however it exits, so a crash leaves
//! nothing to clean up.
//!
//! Another program can ask [`running`] whether the application is up - to
//! start it only when it is not.
//!
//! Install it before every other plugin. A second copy leaves from inside
//! this plugin's `build`, with the process's exit: what plugins installed
//! earlier have done by then - a store opened, migrations run - is done, and
//! their cleanup does not run.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use guinea::app::{AppMeta, Plugin, PluginBuilder};

/// Holds the lock. The application is the running copy while this lives.
pub struct Instance {
    _file: File,
}

/// Takes the lock for `identifier`, or `None` when another copy holds it.
///
/// Tries a few times over a fifth of a second: [`running`] holds the lock for
/// a moment, and a copy started in that moment is not a second one.
pub fn claim(identifier: &str) -> io::Result<Option<Instance>> {
    claim_at(path(identifier)?)
}

/// Whether a copy of `identifier` holds the lock.
pub fn running(identifier: &str) -> bool {
    path(identifier).is_ok_and(|path| matches!(try_claim(&path), Ok(None)))
}

/// Where the lock for `identifier` lives: in the user's profile on Windows,
/// under `~/.cache` elsewhere. Not the runtime directory, which a session
/// has and `su`, cron or a container may not - two copies started with and
/// without it would each find their own lock free.
pub fn path(identifier: &str) -> io::Result<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache"))
    };

    let file = format!("{}.lock", sanitized(identifier));
    base.map(|base| base.join("guinea").join("instances").join(file))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "no per-user directory for the instance lock",
            )
        })
}

fn sanitized(identifier: &str) -> String {
    identifier
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn claim_at(path: PathBuf) -> io::Result<Option<Instance>> {
    for _ in 0..4 {
        if let Some(file) = try_claim(&path)? {
            return Ok(Some(Instance { _file: file }));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(None)
}

/// Opens the lock file, making it and its directory for the owner alone: a
/// lock anyone else could open, anyone else could hold.
fn try_claim(path: &PathBuf) -> io::Result<Option<File>> {
    if let Some(parent) = path.parent() {
        let mut directory = std::fs::DirBuilder::new();
        directory.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut directory, 0o700);
        directory.create(parent)?;
        #[cfg(unix)]
        std::fs::set_permissions(parent, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
    }

    let mut options = OpenOptions::new();
    options.create(true).truncate(false).write(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let file = options.open(path)?;

    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

/// Leaves when another copy of the application is running.
///
/// The identifier is the application's [`AppMeta`] one unless
/// [`named`](Self::named) says otherwise.
#[derive(Default)]
pub struct SingleInstancePlugin {
    identifier: Option<String>,
}

impl SingleInstancePlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks under `identifier` rather than the application's own.
    pub fn named(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }
}

impl Plugin for SingleInstancePlugin {
    const ID: &'static str = "guinea.single-instance";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let identifier = match self.identifier {
            Some(identifier) => identifier,
            None => app
                .try_require::<AppMeta>()
                .map(|meta| meta.identifier.to_string())
                .ok_or_else(|| anyhow::anyhow!("a single instance needs AppMeta or a name"))?,
        };

        let Some(instance) = claim(&identifier)? else {
            tracing::info!(%identifier, "another copy is already running");
            std::process::exit(0);
        };

        app.on_cleanup(move |_| {
            drop(instance);
            Ok(())
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_claim_waits_for_the_first_to_let_go() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let path = dir.path().join("app.lock");

        let first = claim_at(path.clone())
            .expect("claim")
            .expect("the first copy");
        assert!(claim_at(path.clone()).expect("claim").is_none());
        assert!(matches!(try_claim(&path), Ok(None)));

        drop(first);
        assert!(claim_at(path).expect("claim").is_some());
    }

    #[test]
    fn an_identifier_becomes_a_file_name() {
        assert_eq!(
            sanitized("dev.uniproc/guinea devtools"),
            "dev.uniproc_guinea_devtools"
        );
    }
}
