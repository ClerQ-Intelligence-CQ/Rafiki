# Shemasi — app plan (private working notes)

Shemasi = the future Android field app (Expo, APK). Dev/test build
first: progress logs, charts, full feature catalog. This document is
the whole plan so a future build session can start cold.

## 0. What it is

Expo (React Native + TypeScript) Android app. Two modes: normal use
(live twin on device) and dev/test mode (validation runs, progress
logs, charts, exports). First milestone is dev/test only: no store
release, APK via EAS internal distribution.

## 1. Stack (decided, keep)

- Expo SDK + expo-router (file-based nav, five tabs + search + detail).
- Charts: victory-native (lines, bars, gauges) + a custom pulse
  component (live heartbeat strip, Skia if needed later).
- State: Zustand or context + AsyncStorage for logs; SQLite
  (expo-sqlite) once log volume grows.
- Theme: light + dark via ThemeProvider, persisted choice, follows
  system by default.
- Export: CSV + JSON via expo-sharing / StorageAccessFramework;
  PDF summary later (print pipeline, same as course HTMLs).
- Data source v1: local synthetic generator mirroring the engine
  math (same seeds, same regimes as the Rust benches) so the app is
  testable with zero backend. v2 bridge options, in order: on-device
  REST loopback to a Rust core, then UniFFI bindings. No decision
  needed until dev/test mode is done.

## 2. Information architecture: 5 tabs, 150+ tracked items

Catalog counts every tracked thing: 30 SPE features + 30 baselines
+ 30 deviations + 7 classes + 30 twin slots + anomaly records +
  run logs. Each tab owns its slice with search across all of it.

- Tab 1 ACQUIRE (Teka): live stream cards per sensor (accel, GPS,
  baro, mic, screen), pulse strip per stream, confidence readout,
  duty-cycle indicator, sample counters.
- Tab 2 PROCESS (Chuja): feature browser grouped by family (10/9/6/3
  + meta), per-feature sparkline + current value + confidence, window
  fill meters.
- Tab 3 LEARN (Fikiri): baseline cards (mean, variance, stability
  gauge 0-1, n_eff), stability-sorted list, cold/warm state per
  feature, shift-chase demo.
- Tab 4 RECOGNIZE (Tambua): anomaly feed (open/closed), convergence
  detail (contributors + scores), single-vs-multi explainer card,
  deviation leaderboard.
- Tab 5 BECOME (Pacha): twin snapshot card (bytes, completeness,
  freshness), staleness list, feature table with live/stale flags,
  export buttons.

Global search: one box, filters features/baselines/anomalies/runs by
name across all tabs, deep-links to detail. Bottom-tab nav plus
per-tab section headers. Every number tappable to its definition
(glossary reuse from the course).

## 3. Data presentation (heavy, as specified)

- Pulses: live heartbeat strips on every stream card (last N samples,
  scrolling).
- Lines: value-over-time per feature, baseline band overlay
  (mean ± std), deviation markers where z crossed the bar.
- Bars/gauges: stability 0-1 gauges, confidence bars, completeness ring.
- Timeline: anomaly open/close spans on a shared clock with regime
  labels (still/walk/drive/spike phases).
- Heatmap (later): feature × time deviation grid. Listed, not v1.
- Every chart works in light and dark themes (token-based colors,
  never hardcoded hex outside the theme file).

## 4. Dev/test mode (the actual first deliverable)

- Run log: every validation run recorded (engine, regime, seed,
  PASS/FAIL, key numbers, timestamp), list + detail views.
- Progress tracking: per-engine checklists mirroring the repo gates
  (windows, silence, fire, staleness, bytes), streaks, last-green.
- Charts over runs: throughput trend, byte-size trend, stability
  convergence curves per feature.
- Export results: run report as JSON + CSV, share sheet, filename
  includes engine + date + PASS/FAIL.

## 5. Theming

`theme.ts` tokens: background, surface, text, muted, accent, good,
warn, bad, grid. Light + dark sets. No chart or component may use a
literal color. Toggle in header, persisted, system default.

## 6. Milestones (in order)

- M1: scaffold (router, theme, 5 empty tabs, search shell). No data.
- M2: synthetic data layer (seeded regimes matching Rust benches).
- M3: ACQUIRE + PROCESS tabs with pulses, sparklines, browser.
- M4: LEARN tab (gauges, sorted stability, shift demo).
- M5: RECOGNIZE tab (feed, detail, explainer) + dev/test run log.
- M6: BECOME tab (twin card, export CSV/JSON) + progress tracking.
- M7: charts-over-runs, heatmap if cheap, EAS internal APK.
- M8 (later): v2 bridge to real Rust core; store release track.

## 7. Non-goals for now

No store listing, no accounts, no cloud sync, no real sensor
permissions (simulated streams until M8), no video, no marketing
screens. Dev/test build quality first.

## 8. Naming discipline

Public name stays Rafiki. Shemasi never appears in user-visible
strings, store metadata, or release notes. Internal codename only.
