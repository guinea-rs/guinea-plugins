//! The shared secret that lets an application through the handshake.
//!
//! Devtools make a new one every time they start and write it into the user's
//! own profile; an application reads it on every connection attempt. Something
//! that cannot read the user's files cannot connect, and a key left over from
//! an earlier run stops working the moment devtools restart.

use std::io;
use std::path::PathBuf;

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
    let path = path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &secret)?;
    Ok(secret)
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
