# Module 05: Chuja lab, the feature walkthrough (30 minutes)

Goal: know all 30 features and 7 classes by family, and run the
sustained bench yourself.

## 1. The four families

- **Accel (10):** magnitude mean and variance, zero-crossing rate,
  dominant band, jerk (mean absolute derivative), pitch, roll,
  motion energy, peak-to-peak, periodicity. Together they separate
  still, walking, vehicular, and irregular motion.
- **Mic envelope (9):** mean, std, peak, rise and decay times of
  transients, transient count, crest factor, inter-transient spacing,
  impulsiveness. Envelope only, never content.
- **GPS + baro (6):** displacement rate, altitude rate, dwell
  fraction, heading change rate, altitude variance, GPS quality
  (which weights everything downstream).
- **Screen (3):** off fraction, unlocks per hour, longest off stretch.
- **Meta (2):** stability score and aggregate signal quality, derived
  from the families above.

## 2. The seven classes

Motion state, rest state, acoustic environment, transient event,
terrain class, device context, stability band. Each carries the
confidence of the signal underneath it. Low satellite lock means low
confidence on terrain no matter what the classifier says.

## 3. Confidence is load-bearing, not decorative

Every feature carries confidence from signal quality, and every
class carries it forward. Downstream engines weight by it. A loud
room does not confuse the engine; it lowers specific confidences,
which is the correct response to bad signal.

## 4. Try it

```bash
cargo test -p rafiki-spe
cargo run --release -p rafiki-tambua --example bench
```

The bench drives still, walking, and driving regimes and asserts
windows computed in every family, finite statistics, classifications
that vary across regimes, and six-figure throughput.

Experiment `experiment-m05-chuja.zip`: family ablation. Drop one
stream from the input (comment out its samples in the bench loop),
rerun, and record exactly which features go quiet and which classes
lose confidence. The README lists the expected degradation per
dropped stream.

Next: [Module 06](../06-baselines-cold-start/guide.md).
