//! Reads how much memory each watched application holds, from outside it.

use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use guinea_devtools_model::memory::MemorySample;
use guinea_devtools_model::sessions::Incoming;

use crate::Hub;

/// How often each process is read.
const EVERY: Duration = Duration::from_millis(100);

/// Reads every watched process every [`EVERY`] until nobody applies what it
/// reads any more.
pub fn spawn(hub: Arc<Hub>, out: Sender<Incoming>) {
    let _ = std::thread::Builder::new()
        .name("devtools-memory".into())
        .spawn(move || {
            loop {
                std::thread::sleep(EVERY);
                let watched = hub.read().watched();
                for pid in watched {
                    let Some(sample) = read(pid) else {
                        continue;
                    };
                    if out.send(Incoming::Memory(pid, sample)).is_err() {
                        return;
                    }
                }
            }
        });
}

/// What process `pid` holds now; `None` when it cannot be opened.
pub fn read(pid: u32) -> Option<MemorySample> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Performance::QueryPerformanceCounter;
    use windows_sys::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    let process =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, 0, pid) };
    if process.is_null() {
        return None;
    }
    let mut counters = PROCESS_MEMORY_COUNTERS_EX {
        cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let read = unsafe {
        K32GetProcessMemoryInfo(
            process,
            (&raw mut counters).cast::<PROCESS_MEMORY_COUNTERS>(),
            counters.cb,
        )
    };
    let mut qpc = 0i64;
    unsafe {
        QueryPerformanceCounter(&mut qpc);
        CloseHandle(process);
    }

    (read != 0).then_some(MemorySample {
        qpc: qpc as u64,
        private: counters.PrivateUsage as u64,
        working_set: counters.WorkingSetSize as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_process_allocates_shows_in_what_it_holds() {
        let pid = std::process::id();
        let before = read(pid);
        let held = vec![1u8; 64 << 20];
        let after = read(pid);
        drop(std::hint::black_box(held));

        let grew = before.zip(after).map(|(before, after)| {
            (
                after.private.saturating_sub(before.private) >= 60 << 20,
                after.working_set > 0,
                after.qpc > before.qpc,
            )
        });
        assert_eq!(grew, Some((true, true, true)), "{before:?} then {after:?}");
    }
}
