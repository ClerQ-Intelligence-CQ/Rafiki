# Rafiki

<img src="assets/rafiki-logo.jpg" alt="Rafiki - a digital twin" width="360">

> "friend" in Swahili

[![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![language](https://img.shields.io/badge/language-Rust-DEA584.svg)](https://rustup.rs)
[![target](https://img.shields.io/badge/target-4GB_RAM_no_GPU-00A651.svg)](https://github.com/ClerQ-Intelligence-CQ/Rafiki)
[![status](https://img.shields.io/badge/status-five_engines_proven_end_to_end-9B59B6.svg)](https://github.com/ClerQ-Intelligence-CQ/Rafiki)

A research framework for edge-native digital twinning on constrained hardware.

license: Apache-2.0 | language: Rust | target: 4GB RAM / standard CPU / no GPU
status: five engines, wired, proven end to end

What if a device could know you without watching you. Not your photos, not your messages, no camera pointed at your life. Just the signals a phone already has: how it moves, how loud the room is, altitude, when the screen goes dark. What if something small and local could learn what normal looks like for you specifically, and only speak up when something actually breaks from it.

That's what this repo tries to build. No model, no cloud. Five small engines chained together, running on hardware that costs less than a night out.

## The question

Digital twinning, building a living model of a person from ambient data, is usually a cloud problem. Big servers, big GPUs. Rafiki assumes the opposite: most of the world doesn't have that and shouldn't need it for something this basic.

How do you build a living, adaptive model of a person on hardware with no internet, no GPU, and no ML framework, and make it feel intelligent without a single hardcoded rule.

The answer isn't a model. It's five engines, each dumb alone, doing something that looks like understanding together.

## How it works

```
Teka        Chuja          Fikiri          Tambua           Pacha
acquire  →  process    →   learn      →   recognize    →   become
sensors     30+ features   per-person     multi-signal      the
            & classes      baseline       convergence       twin
```

Each engine takes a defined input and produces a defined output. None of them know about the others. They were built and proven one at a time, standalone, before ever being wired together. When they finally were wired together, not one of them needed to change.

## The five engines

```
Teka     signal acquisition    raw sensor input, typed events, adaptive duty cycling
Chuja    signal processing     30+ features and 7 classes, pure DSP, no ML
Fikiri   baseline engine       online Welford stats, decay weighted, per person
Tambua   anomaly detection     multi-signal convergence, not single-signal noise
Pacha    twin state engine     the compact living snapshot, kilobytes not megabytes
```

## Real numbers, from the actual assembled pipeline

```
2400 events in  →  full five-stage chain  →  live twin out
```

| | |
|---|---|
| single mic spike alone: | 0 anomalies fired |
| mic + accel + gps co-spike: | 283 anomalies fired, correct contributors |
| gps held back 400 events: | flagged stale, resolved after |
| anomaly triggered then healed: | reflected in twin, then cleared |

| | |
|---|---|
| end to end throughput: | ~13.3k events/sec |
| final twin snapshot: | 5,066 bytes |
| memory: | flat, 30 tracks, no growth over a sustained run |

The same twin was measured three times, in three different housings: 5,074 bytes running inside the proprietary infrastructure, 5,073 bytes as a standalone fork, 5,066 bytes wired into the full public pipeline. Same shape, same size, no matter where it runs.

## What each engine actually does

Teka turns raw sensor input (accelerometer, GPS, barometer, mic amplitude, screen state) into a typed, timestamped event stream. It never records audio content, only amplitude. It adjusts its own sampling rate based on activity, sampling less when nothing's happening.

Chuja takes that stream and extracts real signal processing features. Motion energy, zero crossing rate, dominant frequency, envelope shape, GPS displacement, altitude variance. Over 30 features and 7 composite classifications, all classical DSP, no models involved.

Fikiri builds a running, per-person baseline for every one of those features using Welford's online algorithm, decay weighted so it tracks drift instead of freezing on old data. It solves the cold start problem without a hardcoded number of days. Baseline stability is derived from the ratio of standard error to variance, so a stable feature earns trust fast and a noisy one earns it slowly, on its own terms.

Tambua compares live values against Fikiri's baseline and looks for convergence. A single feature spiking is noise. Several features spiking together, weighted by how trustworthy each baseline currently is, is worth surfacing. The number of features required to agree scales with how much trustworthy data actually exists, never a fixed constant.

Pacha collects everything upstream into one small, current snapshot. Not a log, not a history, just the state of the person right now, staleness included, anomalies included, all of it small enough to fit in a text message.

## Constraints, held throughout

```
no LLM, no neural network, no ML framework, none, anywhere
no cloud dependency, offline first, always
no fixed thresholds, everything adapts to the person
raw sensor data never persists, only derived state survives
power budget benchmarked per engine, not an afterthought
```

## The cold start problem

Day one, the baseline engine knows nothing. It can't flag anomalies until it has a stable baseline, and there's no clean number of days that works for every feature. Fikiri's answer is to derive stability from the data itself, standard error against variance, so a feature earns trust at its own pace instead of on a clock.

## Quickstart

```
git clone https://github.com/ClerQ-Intelligence-CQ/Rafiki
cd Rafiki/assembly
cargo run --release --example full_pipeline
```

## Contributing

Engine architecture decisions aren't community voted, this is ClerQ Intelligence and Nama Research Labs led research. Contributions to benchmarks, documentation, and language ports are welcome. Open an issue before opening a PR on engine internals.

---

*A ClerQ Intelligence research initiative. Built with Nama Labs proprietary AI infrastructure.*
