//! Loading the tap into a running process. This half runs in devtools.
//!
//! WinUI 3 exports `InitializeXamlDiagnosticsEx` from the target's own
//! `Microsoft.Internal.FrameworkUdk.dll`, not from a system DLL, so devtools
//! load that same file and call it. The target's XAML runtime then loads the
//! tap itself, over an endpoint named `WinUIVisualDiagConnection<N>` - one per
//! XAML thread, counted from one.

use std::path::Path;

use windows_core::{GUID, HRESULT, PCWSTR, s};

use crate::CLSID;
use crate::bindings::{
    CloseHandle, CreateToolhelp32Snapshot, GetProcAddress, LoadLibraryW, MODULEENTRY32W,
    Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
};

const RUNTIME: &str = "Microsoft.Internal.FrameworkUdk.dll";
const ENDPOINTS: u32 = 16;

type Initialize = unsafe extern "system" fn(PCWSTR, u32, PCWSTR, PCWSTR, GUID, PCWSTR) -> HRESULT;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn text(raw: &[u16]) -> String {
    let end = raw.iter().position(|&unit| unit == 0).unwrap_or(raw.len());
    String::from_utf16_lossy(&raw[..end])
}

/// Where the process's WinUI runtime is loaded from, if it has one.
pub fn winui_runtime(pid: u32) -> Option<String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE as u32, pid) };
    if snapshot.is_null() || snapshot as isize == -1 {
        return None;
    }

    let mut entry = MODULEENTRY32W {
        dwSize: size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = None;

    let mut more = unsafe { Module32FirstW(snapshot, &mut entry) }.as_bool();
    while more {
        if text(&entry.szModule).eq_ignore_ascii_case(RUNTIME) {
            found = Some(text(&entry.szExePath));
            break;
        }
        more = unsafe { Module32NextW(snapshot, &mut entry) }.as_bool();
    }

    let _ = unsafe { CloseHandle(snapshot) };
    found
}

/// Loads the tap at `dll` into process `pid`.
pub fn inject(pid: u32, dll: &Path) -> Result<(), String> {
    let runtime = winui_runtime(pid).ok_or("the process has no WinUI 3 runtime loaded")?;

    let library = unsafe { LoadLibraryW(PCWSTR(wide(&runtime).as_ptr())) };
    if library.is_null() {
        return Err(format!("cannot load {runtime}"));
    }
    let export = unsafe { GetProcAddress(library, s!("InitializeXamlDiagnosticsEx")) }
        .ok_or("the WinUI runtime has no InitializeXamlDiagnosticsEx")?;
    let initialize: Initialize = unsafe { std::mem::transmute(export) };

    let dll = wide(&dll.to_string_lossy());
    let empty = wide("");
    let mut last = HRESULT(0);

    for n in 1..=ENDPOINTS {
        let endpoint = wide(&format!("WinUIVisualDiagConnection{n}"));
        last = unsafe {
            initialize(
                PCWSTR(endpoint.as_ptr()),
                pid,
                PCWSTR(empty.as_ptr()),
                PCWSTR(dll.as_ptr()),
                CLSID,
                PCWSTR(empty.as_ptr()),
            )
        };
        if last.is_ok() {
            return Ok(());
        }
    }

    Err(format!(
        "no XAML diagnostics endpoint took the tap: {last:?}"
    ))
}
