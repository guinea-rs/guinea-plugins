//! The async runtime the application runs on, as tokio's own counters tell
//! it.

use guinea_devtools_protocol::{RuntimeReading, WorkerReading};

/// What the runtime `handle` names had done by now, stamped `at`.
pub fn read(handle: &tokio::runtime::Handle, at: u64) -> RuntimeReading {
    let metrics = handle.metrics();
    RuntimeReading {
        at,
        alive_tasks: metrics.num_alive_tasks() as u64,
        queued: metrics.global_queue_depth() as u64,
        workers: (0..metrics.num_workers())
            .map(|worker| WorkerReading {
                busy_us: metrics.worker_total_busy_duration(worker).as_micros() as u64,
                parks: metrics.worker_park_count(worker),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn a_worker_that_ran_a_task_says_how_long_and_a_waiting_task_is_alive() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .map_err(|error| error.to_string());
        let busy = |reading: &RuntimeReading| {
            reading
                .workers
                .iter()
                .map(|worker| worker.busy_us)
                .sum::<u64>()
        };
        let reading = runtime.map(|runtime| {
            runtime.spawn(std::future::pending::<()>());
            let spun = runtime.spawn(async {
                let started = Instant::now();
                while started.elapsed() < Duration::from_millis(60) {
                    std::hint::spin_loop();
                }
            });
            let _ = runtime.block_on(spun);
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let reading = read(runtime.handle(), 7);
                if busy(&reading) >= 50_000 || Instant::now() > deadline {
                    break reading;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });

        let seen = reading.map(|reading| {
            (
                reading.at,
                reading.workers.len(),
                busy(&reading) >= 50_000,
                reading.alive_tasks >= 1,
            )
        });
        assert_eq!(seen, Ok((7, 2, true, true)));
    }
}
