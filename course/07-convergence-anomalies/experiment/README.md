# Module 07 experiment: spike discipline

## Objective

Prove silence and fire are both structural, then try to break them.

## Steps

1. `cargo run --release -p rafiki-tambua --example bench`
   (expect single 0, multi 314 with accel/mic/gps contributors).
2. In the bench source, extend the single-spike phase from 150 to
   1500 steps. Rerun.
3. Restore. Rerun to confirm baseline numbers return.

## Expected outputs

- Baseline: single 0, multi 314, PASS.
- Extended single: still 0. Duration does not matter because the
  rule counts families, not samples or seconds.
- Restored: identical to baseline (deterministic seeds).

## Questions

(a) Why doesn't a longer single spike eventually fire?
(b) What would happen if classifications were allowed to vote?
(Answer: one mic spike plus its acoustic_env echo would read as two
signals. That exact bug was caught and fixed during validation.)
(c) Where is the N formula, and what inputs does it take?
