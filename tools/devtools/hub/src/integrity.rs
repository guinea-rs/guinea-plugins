//! A process's integrity level: how far Windows trusts it, and so whether
//! devtools may take a file from it.

use std::ffi::c_void;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TOKEN_MANDATORY_LABEL,
    TOKEN_QUERY, TokenIntegrityLevel,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// This process's level.
pub fn own() -> Option<u32> {
    level(unsafe { GetCurrentProcess() })
}

/// Process `pid`'s level, when it can be asked.
pub fn of(pid: u32) -> Option<u32> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return None;
    }
    let found = level(process);
    unsafe { CloseHandle(process) };
    found
}

fn level(process: HANDLE) -> Option<u32> {
    let mut token: HANDLE = std::ptr::null_mut();
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return None;
    }

    let mut label = [0u64; 16];
    let mut written = 0;
    let read = unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            label.as_mut_ptr().cast::<c_void>(),
            size_of_val(&label) as u32,
            &mut written,
        )
    };
    unsafe { CloseHandle(token) };
    if read == 0 {
        return None;
    }

    let label = unsafe { &*label.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
    let sid = label.Label.Sid;
    let count = unsafe { *GetSidSubAuthorityCount(sid) };
    let last = count.checked_sub(1)?;
    Some(unsafe { *GetSidSubAuthority(sid, u32::from(last)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_process_is_as_trusted_as_itself() {
        let ours = own().expect("this process has a level");
        assert!(ours >= 0x1000, "at least low: {ours:#x}");
        assert_eq!(of(std::process::id()), Some(ours));
        assert_eq!(of(u32::MAX), None);
    }
}
