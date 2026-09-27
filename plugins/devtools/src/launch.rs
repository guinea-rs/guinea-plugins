//! Starts devtools for an application that asked for them.
//!
//! The binary is `GUINEA_DEVTOOLS_BIN` when that is set, and `guinea-devtools`
//! from the `PATH` otherwise - which is where `cargo install` puts it. It is
//! started only when no devtools hold their single-instance lock, and devtools
//! that do start as a second copy leave on their own.

use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Child, Command, Stdio};

pub const VARIABLE: &str = "GUINEA_DEVTOOLS_BIN";
const BINARY: &str = "guinea-devtools";

pub fn start() {
    if guinea_plugin_single_instance::running(guinea_devtools_protocol::IDENTIFIER) {
        return;
    }

    let program = std::env::var_os(VARIABLE).unwrap_or_else(|| OsString::from(BINARY));
    let shown = program.to_string_lossy().into_owned();

    match spawn(&program) {
        Ok(mut child) => {
            tracing::info!(program = %shown, pid = child.id(), "started devtools");
            let _ = std::thread::Builder::new()
                .name("guinea-devtools-child".into())
                .spawn(move || child.wait());
        }
        Err(error) => tracing::warn!(
            program = %shown,
            %error,
            "devtools could not be started: install them with `cargo install --path tools/devtools` \
             in guinea-plugins, or set {VARIABLE} to the binary"
        ),
    }
}

fn spawn(program: &OsStr) -> io::Result<Child> {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;

        if let Ok(child) = command
            .creation_flags(CREATE_NO_WINDOW | CREATE_BREAKAWAY_FROM_JOB)
            .spawn()
        {
            return Ok(child);
        }
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.spawn()
}
