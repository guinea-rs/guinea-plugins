# Progressive scatter native diagnostic, 2026-11-07

Three same-binary pairs, Windows native Uniproc Activity probe, 100,000 synthetic
points, 200 virtualized rows, new snapshots every 250 ms for four seconds and a
subsequent drain. No builds during measurement. The diagnostic window uses the
consumer's no-activation probe; no user window or pointer was controlled.

Median steady maximum UI callback gap: 320.711 -> 47.915 ms.
Median p95 UI callback gap: 212.612 -> 23.752 ms.
Median native CompositionTarget.Rendered: 100.867 -> 87.503 ms.
Processed updates: median 13 -> 15.

Progressive jobs used a soft four-millisecond submission budget. Observed maximum
chunk wall time across these runs: 7.154 ms. Ordinary live updates caused zero
cancellations. After draining, the front revision matched the requested revision
in every progressive run. Detailed timings and generations are in native-summary.json;
raw aggregate observations are the six CSVs. No per-process user data is included.

This confirms reduced UI stalls in this diagnostic, not lower total CPU, FPS or
physical DWM presentation latency. Cold device/native mount costs remain. The
four-millisecond setting is not a hard upper bound, including EndDraw.

The measured prototype has the same scheduler and rendering as this commit.
Final changes split BeginDraw from other CPU wall times in diagnostics and add
regressions; they do not change cursor traversal, job scheduling or the budget.
Owner-side WARP comparisons cover 1/1.5/2 DPI with bounded premultiplied-alpha
compositing roundoff, and regressions cover pending snapshots, cancellation,
brush pinning, and front-consistent hit testing.

Local release checks: 146 workspace tests passed, 3 explicitly ignored;
strict clippy with default and all features passed; standalone DevTools all-target
clippy passed. Dependency audits passed with the explicit maintenance-only,
time-bounded cfg(loom)/pin-utils exception recorded in deny.toml. No vulnerability
advisory was disabled. Remote CI, tagging and publication require authorization
and have not run for this commit.
