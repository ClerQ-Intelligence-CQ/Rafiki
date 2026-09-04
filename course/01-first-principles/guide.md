# Module 01: First principles (30 minutes)

Goal: understand the problem, the idea, and every word the rest of
the course uses. No code in this lesson except the glossary page.

## 1. The problem, plainly

Most health and body tracking assumes a flagship phone, a data plan,
and a cloud account. Most of the world has none of those and should
not need them for something as basic as knowing whether today looks
like your normal days. The target: a living model of one person that
runs fully offline, on ordinary hardware, with no model training.

## 2. Words you need (full glossary in `glossary.html`)

- **Sensor:** hardware that measures the world (accelerometer, GPS,
  barometer, microphone, screen state).
- **Event / event stream:** one timestamped reading, then the
  sequence of them. Everything downstream reads streams, never raw
  hardware.
- **Feature:** a computed number describing a window of events
  (mean, variance, displacement rate). Chuja produces 30 of them.
- **Baseline:** what a feature normally looks like for this person
  (running mean and variance). Fikiri keeps one per feature.
- **Stability:** how much the baseline itself can still move, from 0
  to 1. Derived from standard error against variance. New features
  start near 0; steady ones climb toward 1.
- **Deviation:** how far a live value sits from its baseline, in
  units of that feature's own variance (z-score style).
- **Convergence:** deviations co-occurring across independent sensor
  families inside one time window. One loud stream is noise; several
  together is signal.
- **Anomaly:** a fired convergence event: what moved, how far, how
  confident, since when.
- **Twin:** the compact current snapshot of everything above. About
  5 kilobytes. Not a history, not a log.
- **Duty cycling:** sampling fast when active, slow when quiet, to
  protect the battery.
- **Offline-first:** no network calls after setup. If a step needs
  the network, the guide says so.
- **Deterministic:** same input, same output. No sampling luck.

## 3. Why no models

A model learns population averages and needs training data, GPUs,
and updates. This system learns one person from their own stream
using closed-form statistics (running means, variances, counts).
Nothing trains, nothing downloads, everything runs on a $30-class
device. That is the whole bet: architecture instead of models.

## 4. The five engines in one paragraph each

**Teka** turns raw sensor input into typed, timestamped events and
scores each with confidence from signal quality. Quiet sensors rest,
motion wakes them.

**Chuja** extracts real signal processing features from those
events: motion energy, zero crossings, dominant frequency bands,
envelope shape, displacement, altitude variance. Classical DSP only.

**Fikiri** keeps a running per-person baseline per feature with
decay-weighted Welford statistics, so baselines track drift. Cold
start is solved mathematically: stability comes from the data, never
from a hardcoded number of days.

**Tambua** scores live values against baselines and fires only on
multi-signal convergence, with the required count scaling to
trustworthy signal. Single spikes stay silent, by construction.

**Pacha** assembles the twin: current values, baselines, deviations,
freshness, liveness, open anomalies. About 5 kilobytes, serializable,
queryable.

## 5. Try it (no code)

Open `glossary.html` in a browser. Hover every term. Then open the
verb-style explorer mindset for Module 02: you will meet each sensor
with its units and its noise.

Next: Module 02 (Signals 101) — *under construction, follows this module.*
