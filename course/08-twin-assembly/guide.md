# Module 08: twin and assembly (30 minutes)

Goal: see all five engines run as one pipeline, and understand what
the twin is and is not.

## 1. What the twin holds

One fixed slot per tracked feature: current value, baseline mean,
stability, deviation, last-updated timestamp, live and stale flags.
Plus an optional open anomaly (what, confidence, since when), an
overall completeness share, and a freshness timestamp. Thirty tracks
serialize to about 5 kilobytes. Not a log, not a history: current
state only.

## 2. Staleness is honest metadata

A feature whose data stopped arriving gets flagged stale after the
horizon, and completeness drops to say so. The twin never pretends
old data is current. You will hold GPS back yourself and watch the
flag appear, then resolve when the stream returns.

## 3. Assembly changes nothing

The wiring layer calls engine outputs into engine inputs, nothing
more. Proof it adds no logic of its own: the assembled run
reproduces every per-engine proof (silence, fire, staleness,
reflect/clear) with zero engine modifications. If wiring ever needs
an engine changed, that is reported as a finding, not slipped in.

## 4. Try it

```bash
cargo run --release -p rafiki-assembly --example full_pipeline
```

Expect: single 0, multi in the hundreds with sensor-family
contributors, stale flagged and resolved, anomaly reflected then
cleared, twin about 5 kilobytes, footprint flat at 30 tracks.

Experiment `experiment-m08-assembly.zip`: run it, record all eight
lines, then halve the convergence window and rerun. Note which
numbers move and which do not, and explain why throughput changes
but the twin byte size does not.

Next: [Module 09](../09-field-use/guide.md).
