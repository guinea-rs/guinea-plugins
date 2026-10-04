//! One moment read on every clock a profile is stitched from.

use guinea_devtools_protocol::ClockAnchor;

/// Reads every clock now, on the calling thread.
pub fn anchor() -> ClockAnchor {
    let trace_before = guinea_core::trace::now();
    let qpc = os::qpc();
    #[cfg(feature = "profiling")]
    let puffin_ns = Some(puffin::now_ns());
    #[cfg(not(feature = "profiling"))]
    let puffin_ns = None;
    let trace_after = guinea_core::trace::now();

    ClockAnchor {
        qpc,
        qpc_frequency: os::qpc_frequency(),
        trace_us: ((trace_before + trace_after) / 2).as_micros() as u64,
        puffin_ns,
        ui_thread: os::thread(),
    }
}

#[cfg(windows)]
mod os {
    windows_link::link!("kernel32.dll" "system" fn QueryPerformanceCounter(count: *mut i64) -> i32);
    windows_link::link!("kernel32.dll" "system" fn QueryPerformanceFrequency(frequency: *mut i64) -> i32);
    windows_link::link!("kernel32.dll" "system" fn GetCurrentThreadId() -> u32);

    pub fn qpc() -> u64 {
        let mut count = 0;
        unsafe { QueryPerformanceCounter(&mut count) };
        count as u64
    }

    pub fn qpc_frequency() -> u64 {
        let mut frequency = 0;
        unsafe { QueryPerformanceFrequency(&mut frequency) };
        frequency as u64
    }

    pub fn thread() -> u64 {
        u64::from(unsafe { GetCurrentThreadId() })
    }
}

#[cfg(not(windows))]
mod os {
    pub fn qpc() -> u64 {
        0
    }

    pub fn qpc_frequency() -> u64 {
        0
    }

    pub fn thread() -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[cfg(windows)]
    mod os {
        windows_link::link!("kernel32.dll" "system" fn QueryPerformanceCounter(count: *mut i64) -> i32);
        windows_link::link!("kernel32.dll" "system" fn GetCurrentThreadId() -> u32);

        pub fn qpc() -> u64 {
            let mut count = 0;
            unsafe { QueryPerformanceCounter(&mut count) };
            count as u64
        }

        pub fn thread() -> u64 {
            u64::from(unsafe { GetCurrentThreadId() })
        }
    }

    fn micros(since: Duration) -> u64 {
        since.as_micros() as u64
    }

    #[test]
    fn every_clock_is_read_at_one_moment() {
        guinea_core::trace::now();
        std::thread::sleep(Duration::from_millis(2));

        let trace_before = micros(guinea_core::trace::now());
        #[cfg(windows)]
        let qpc_before = os::qpc();
        #[cfg(feature = "profiling")]
        let puffin_before = puffin::now_ns();

        let anchor = anchor();

        #[cfg(feature = "profiling")]
        let puffin_after = puffin::now_ns();
        #[cfg(windows)]
        let qpc_after = os::qpc();
        let trace_after = micros(guinea_core::trace::now());

        assert!(
            (trace_before..=trace_after).contains(&anchor.trace_us),
            "trace {} outside {trace_before}..={trace_after}",
            anchor.trace_us
        );

        #[cfg(windows)]
        {
            assert!(
                (qpc_before..=qpc_after).contains(&anchor.qpc),
                "qpc {} outside {qpc_before}..={qpc_after}",
                anchor.qpc
            );
            assert!(anchor.qpc_frequency > 0, "no qpc frequency");
            assert_eq!(anchor.ui_thread, os::thread());
        }

        #[cfg(feature = "profiling")]
        {
            let puffin = anchor.puffin_ns.expect("puffin is compiled in");
            assert!(
                (puffin_before..=puffin_after).contains(&puffin),
                "puffin {puffin} outside {puffin_before}..={puffin_after}"
            );
        }
        #[cfg(not(feature = "profiling"))]
        assert_eq!(anchor.puffin_ns, None);
    }
}
