//! Where devtools answer over HTTP, and the token they want.
//!
//! Devtools write both next to the endpoint's key when they start, in the
//! user's own profile: something that cannot read the user's files cannot ask.

use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Access {
    /// `http://127.0.0.1:47385`
    pub url: String,
    /// Sent as `Authorization: Bearer <token>`.
    pub token: String,
}

pub fn path() -> io::Result<PathBuf> {
    let key = guinea_devtools_protocol::key::path()?;
    Ok(key.with_file_name("devtools.http.json"))
}

impl Access {
    pub fn write(&self) -> io::Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        guinea_devtools_protocol::key::write_private(&path()?, text.as_bytes())
    }

    pub fn read() -> io::Result<Self> {
        let text = std::fs::read_to_string(path()?)?;
        serde_json::from_str(&text)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}
