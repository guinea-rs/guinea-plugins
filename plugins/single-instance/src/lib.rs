//! Keeps one copy of an application running per user.
//!
//! ```no_run
//! use guinea_plugin_single_instance::{SingleInstanceConfig, SingleInstancePlugin};
//! fn app(app: &mut guinea::app::FeatureBuilder) -> anyhow::Result<()> {
//!     let meta = guinea::app::AppMeta::new("app", "dev.example.app", "0.1.0", "example");
//!     app.plugin::<SingleInstancePlugin>(SingleInstanceConfig::new(&meta))?;
//!     Ok(())
//! }
//! ```
//!
//! The first copy holds a lock on a file named after the application's
//! identifier, in the user's own profile, for as long as it runs. A copy that
//! finds the lock taken stops before it opens a window: this plugin's `build`
//! returns [`Stop`](guinea::app::Stop), the plugins installed before it are
//! cleaned up again, and `run` returns `Ok`. The operating system drops the
//! lock when the holder exits, however it exits, so a crash leaves nothing to
//! clean up.
//!
//! Another program can ask [`running`] whether the application is up - to
//! start it only when it is not.
//!
//! Where it sits among the plugins is up to the application. What the ones
//! before it do that their cleanup does not undo - a migration run - a second
//! copy does too.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use guinea::app::{AppMeta, PluginBuilder};

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

/// `identifier` as a file name, every byte that is not `[A-Za-z0-9._-]`
/// written as `%XX`, so that two identifiers never share one.
fn sanitized(identifier: &str) -> String {
    use std::fmt::Write;

    let mut name = String::with_capacity(identifier.len());
    for byte in identifier.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_') {
            name.push(byte as char);
        } else {
            let _ = write!(name, "%{byte:02X}");
        }
    }
    name
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

/// Stops the application when another copy of it is running.
///
/// Construct from [`AppMeta`] or use [`named`](Self::named) for an explicit identifier.
pub struct SingleInstanceConfig {
    identifier: String,
}

impl SingleInstanceConfig {
    pub fn new(meta: &AppMeta) -> Self {
        Self {
            identifier: meta.identifier.to_string(),
        }
    }

    /// Locks under `identifier` rather than the application's own.
    pub fn named(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
        }
    }
}

guinea::plugin! { pub SingleInstancePlugin {} }

#[guinea::builds]
fn single_instance(
    app: &mut PluginBuilder,
    config: SingleInstanceConfig,
) -> anyhow::Result<SingleInstancePlugin> {
    let identifier = config.identifier;
    let Some(instance) = claim(&identifier)? else {
        tracing::info!(%identifier, "another copy is already running");
        return Err(guinea::app::Stop.into());
    };

    app.on_cleanup(move || {
        drop(instance);
        Ok(())
    });
    Ok(SingleInstancePlugin)
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

    guinea::plugin! { Earlier {} }

    #[guinea::builds]
    fn earlier(
        app: &mut PluginBuilder,
        cleaned: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> anyhow::Result<Earlier> {
        app.on_cleanup(move || {
            cleaned.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });
        Ok(Earlier)
    }

    #[test]
    fn a_second_copy_stops_and_what_came_before_it_is_cleaned_up() {
        let identifier = format!("guinea-plugins.single-instance-test.{}", std::process::id());
        let first = claim(&identifier).expect("claim").expect("the first copy");
        let cleaned = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let result = (|| -> anyhow::Result<guinea::app::TestApp> {
            let mut app = guinea::app::TestApp::new();
            app.plugin::<Earlier>(cleaned.clone())?;
            app.plugin::<SingleInstancePlugin>(SingleInstanceConfig::named(&identifier))?;
            Ok(app)
        })();
        let error = result.err().expect("the second copy stops");
        assert!(error.is::<guinea::app::Stop>());
        assert!(cleaned.load(std::sync::atomic::Ordering::SeqCst));
        drop(first);
        let _ = std::fs::remove_file(path(&identifier).expect("a path"));
    }

    #[test]
    fn an_identifier_becomes_a_file_name_of_its_own() {
        assert_eq!(
            [
                sanitized("dev.uniproc.guinea-devtools_1"),
                sanitized("dev.uniproc/guinea devtools"),
                sanitized("мой.редактор"),
            ],
            [
                "dev.uniproc.guinea-devtools_1",
                "dev.uniproc%2Fguinea%20devtools",
                "%D0%BC%D0%BE%D0%B9.%D1%80%D0%B5%D0%B4%D0%B0%D0%BA%D1%82%D0%BE%D1%80",
            ]
        );
        assert_ne!(sanitized("a/b"), sanitized("a_b"));
        assert_ne!(sanitized("мой.редактор"), sanitized("мой.терминал"));
    }
}
