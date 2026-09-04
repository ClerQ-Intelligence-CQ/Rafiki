# Module 06 experiment: chase the shift

## Objective

Prove the baseline tracks a moving world instead of freezing on
history.

## Steps

1. `cargo run --release -p rafiki-fikiri --example bench` (expect
   PASS; record the mic baseline before/after the deliberate shift).
2. In the engine source, find the decay constant (500) and halve it.
3. Rerun. Record the new before/after and the stability numbers.
4. Restore 500. Rerun to confirm baseline numbers return.

## Expected outputs

- Default run: mic baseline moves most of the way across the shift
  (measured 42.65 to 65.32 on a 44 to 70 move).
- Halved decay: tracks faster and further, stability lower at equal
  counts (fresher memory means less evidence behind it).
- Restored run: identical to the first (deterministic seeds).

## Questions

(a) Why does halving decay lower stability at equal sample counts?
(b) What breaks if decay is removed entirely (pure Welford)?
(Answer: the baseline freezes more rigid every day; drift becomes
invisible.)
(c) Where is the "no cutoff" property enforced in code? (Answer: no
threshold on n anywhere in the scoring path; S is continuous.)
