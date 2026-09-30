//! The shared secret that lets an application through the handshake.
//!
//! Devtools make a new one every time they start and write it into the user's
//! own profile; an application reads it on every connection attempt. Something
//! that cannot read the user's files cannot connect, and a key left over from
//! an earlier run stops working the moment devtools restart.

use std::io;
use std::path::{Path, PathBuf};

use ogurpchik::auth::handshake::HandshakeMode;

const LEN: usize = 32;

pub fn path() -> io::Result<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    };
    base.map(|base| base.join("guinea").join("devtools.key"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "no per-user directory for the devtools key",
            )
        })
}

/// Makes a fresh key, replacing any earlier one. For devtools.
pub fn create() -> io::Result<Vec<u8>> {
    let mut secret = vec![0u8; LEN];
    getrandom::fill(&mut secret).map_err(|error| io::Error::other(error.to_string()))?;
    write_private(&path()?, &secret)?;
    Ok(secret)
}

/// Writes `contents` to `path` for this user's eyes only, whole or not at
/// all: into a file beside it that only the owner may read, then renamed over
/// it. The directory is made owner-only when it is made; on Windows both sit
/// in the user's own profile and take its protection.
pub fn write_private(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "a file needs a directory to be in",
        )
    })?;
    let mut directory = std::fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut directory, 0o700);
    directory.create(parent)?;
    #[cfg(unix)]
    std::fs::set_permissions(parent, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;

    let mut unique = [0u8; 8];
    getrandom::fill(&mut unique).map_err(|error| io::Error::other(error.to_string()))?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let staged = parent.join(format!(".{name}.{:016x}", u64::from_le_bytes(unique)));

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);

    let written = options.open(&staged).and_then(|mut file| {
        io::Write::write_all(&mut file, contents)?;
        file.sync_all()
    });
    let renamed = written.and_then(|()| std::fs::rename(&staged, path));
    if renamed.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    renamed
}

/// The key devtools wrote. For an application.
pub fn read() -> io::Result<Vec<u8>> {
    let secret = std::fs::read(path()?)?;
    if secret.len() == LEN {
        Ok(secret)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the devtools key has the wrong length",
        ))
    }
}

pub fn handshake(secret: Vec<u8>) -> HandshakeMode {
    HandshakeMode::hmac(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_private_file_replaces_the_last_and_leaves_nothing_beside_it() {
        let home = tempfile::tempdir().expect("a directory");
        let path = home.path().join("guinea").join("devtools.key");

        write_private(&path, b"first").expect("written");
        write_private(&path, b"second").expect("written again");

        assert_eq!(std::fs::read(&path).expect("read"), b"second");
        let beside: Vec<_> = std::fs::read_dir(path.parent().expect("a parent"))
            .expect("listed")
            .collect();
        assert_eq!(beside.len(), 1, "no staged file left over");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode =
                |path: &Path| std::fs::metadata(path).expect("there").permissions().mode() & 0o777;
            assert_eq!(mode(&path), 0o600);
            assert_eq!(mode(path.parent().expect("a parent")), 0o700);
        }
    }
}
