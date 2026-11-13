# Line-chart diagnostics, 2026-11-13

Release candidate based on guinea-plugins v0.35.2; Canvas/Reactor versions unchanged.
Three charts: main 800x180 and two sidebars 220x52; two synthetic series each,
monotone interpolation and dashed second stroke. Native probes use 61 samples per
series; the separate offscreen diagnostic uses 60.

## Native WinUI consumer probe

Uniproc's no-activation hosting ran pixel, smooth, paused and unmounted phases,
12 seconds each, with publications continuing during pause/unmount. Sources were
hashed before/after and unchanged. Observer buffer capacity 20,000; contiguous
sequence validation and overflow counters were enabled. Process CPU was sampled
over the central seven seconds on a 12-logical-processor machine.

| Limit | Draws per chart in smooth phase | Sequence gaps / overflow | Paused / unmounted draws | Process CPU, machine |
|---|---:|---:|---:|---:|
| 30 FPS | 361 | 0 / 0 | 0 / 0 | 1.245% |
| 60 FPS | 721 | 0 / 0 | 0 / 0 | 1.508% |

Counts include the initial static draw. Aggregate CSVs are in native-fps-30 and
native-fps-60. These are one pass each, not a statistically established speedup.
Earlier uncapped probes drew 1,730–1,732 frames per chart at approximately 144 Hz
and measured 2.503–4.663% process CPU over three trials. Capping demonstrably
reduces admitted surface draws; CPU outside the paint stages remains noisy.
The 30-FPS run measured 1.370% CPU while paused with zero paints, versus 0.055%
in the 60-FPS run. This anomaly is unattributed: it does not establish a Rendering
subscription leak or a steady pause cost. Surface timing sums are not full CPU
attribution. The 16-ms heartbeat was a chained timer, not a vsync/presentation
measurement; see CSVs for its observed gaps.

Probe driver, full frame sequences and unchanged-source hashes are retained by
the Uniproc project under diagnostics/chart-live/20261113-capped. Only synthetic
aggregate results are copied here; no real application/process data is included.
Visual 30/60 comparison is a separate consumer window, not a quantified result.

## Offscreen path-cache diagnostic

The ignored line_cache_diagnostic test uses the default hardware adapter (no WARP
fallback), release profile, two 60-point curves, fixed Y range, 60-second window,
rounded corners and 96 DPI. Each case runs 660 unpaced virtual frames; the first
60 are warm-up. Cache invalidation every 60 frames models a one-second publication
cadence. Values stay fixed while time advances. The rebuild baseline recreates
paths, brushes and styles every frame. No window is opened by this test.

Three consecutive same-binary trials are recorded in offscreen-1/2/3.txt:

| Surface | Per-frame rebuild mean wall time | Cached mean wall time |
|---|---:|---:|
| Main | 496–548 us | 459–554 us |
| Sidebar | 188–207 us | 175–190 us |

Geometry builds are 660 versus 11 in every case. Cached EndDraw-tail p95 ranges
628–874 us for main and 263–296 us for sidebar. Finish includes EndDraw/wrapper
work, not GPU completion. One cached main trial is slower than its baseline:
these results establish avoided geometry work, not a reliable frame-time gain.
Offscreen targets do not measure WinUI CanvasImageSource overhead or presentation.
source-hashes.json records that earlier measured source snapshot, before observer
and frame-cap additions. It is not a hash of the final release candidate.

## Checks and interpretation

Widget checks: 65 tests passed, one diagnostic explicitly ignored; strict all-target
clippy passed. Regressions cover irregular monotone curves without overshoot,
cached fractional-translation raster equivalence, stroke indexing, pause/resume,
per-frame observer sequencing and deterministic rate gates at 24/30/60/144/240 Hz.
Switching 30/60 preserves data and the elapsed-time clock. Existing Scatter tests
remain green. Full release-candidate all-feature workspace tests passed: 153 tests, four
explicitly ignored. Strict default/all-feature workspace clippy, formatting in
both workspaces and both dependency audits passed. Remote CI is not yet run. No claim here measures GPU completion or physical presentation.
