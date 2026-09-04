# Rafiki

> "friend" in Swahili

A research framework for edge-native digital twinning on constrained hardware.

- license: Apache-2.0
- language: Rust
- target: 4GB RAM / standard CPU / no GPU
- status: research in progress

---

Digital twinning assumes cloud. It assumes GPU. It assumes enterprise hardware.

Rafiki does not.

> How do you build a living, adaptive personal model of a human being on hardware with no internet, no GPU, and no machine learning framework — and make it feel genuinely intelligent without a single hardcoded rule?

The answer is not a model. It is an architecture.

---

## How it works

```
                 acquire         process          learn
sensor input  ---------->  feature vec  -------->  personal baseline
                 (SAE)                   (SPE)          (BE)
                                                         |
                                                         v
                                                 anomaly detection
                                                      (ADE)
                                                         |
                                                         v
                                                    twin state
                                                      (TSE)
```

Five engines. Each takes a defined input, produces a defined output, knows nothing about the others. Assembly comes only after each engine is individually validated and benchmarked.

No LLM. No neural network. No ML framework. No fixed thresholds. The system learns you — not a population average of you.

---

## What's in the box

```
SAE   signal acquisition     raw sensor input → typed timestamped event stream
SPE   signal processing      event stream → feature vectors with confidence scores
BE    baseline engine        online adaptive personal baseline — no batch retrain ever
ADE   anomaly detection      multi-signal convergence — single signal alone is noise
TSE   twin state engine      compact serializable living twin — kilobytes, not megabytes
```

---

## Real output (SAE v0.1, simulated input)

```
[00:00:00.000] SAE::init  duty_cycle=adaptive  sensors=[accel, gps, baro, mic_amplitude]
[00:00:00.012] EVENT  accel    { x: 0.02, y: 0.01, z: 9.81,  ts: 1722470400012 }
[00:00:00.012] EVENT  baro     { altitude_m: 54.3,            ts: 1722470400012 }
[00:00:00.015] EVENT  mic_env  { amplitude_db: 41.2,          ts: 1722470400015 }
// amplitude envelope only — content never recorded, never stored
[00:00:01.200] DUTY   low_activity detected → sampling rate -50%

peak_mem: 1.2MB   steady_state: 0.8MB   cpu_avg: 0.3%
```

---

## Constraints

Architectural commitments, not preferences.

```
no LLM, neural network, or ML framework — none, ever
no cloud dependency — offline-first, always
no fixed thresholds — everything adapts to the individual
raw sensor data never persists — only derived state survives
power budget is benchmarked per engine, not an afterthought
```

---

## The cold start problem

The baseline engine has no data on day one. It cannot surface meaningful anomalies until a stable personal baseline exists. What is the mathematically principled minimum collection period before BE output is trustworthy — and what does the system surface during that period?

Do not hardcode a number. Derive it from the statistical properties of the data.

This is the first open research question Rafiki must answer.

---

## Research standard

Every engine ships with a clean Rust implementation, documented design rationale, benchmark results on the 4GB no-GPU target, peak and steady-state memory profile, power estimate where measurable, known limitations, and a clear interface contract.

Public research must be readable and useful to someone who was not in the room.

---

## Quickstart

```
git clone https://github.com/ClerQ-Intelligence-CQ/Rafiki
cd rafiki/engines/sae
cargo run --release --example simulate
```

---

## Contributing

Engine architecture decisions are not community-voted. Contributions to benchmarks, documentation, and language ports are welcome — open an issue before opening a PR on engine internals.

---

*A ClerQ Intelligence research initiative.*

---

Built with Nama Labs proprietary AI infrastructure.
