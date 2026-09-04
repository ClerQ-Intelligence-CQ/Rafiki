# Module 05 experiment: family ablation

## Objective

Prove each family earns its place: remove one stream, measure exactly
what degrades.

## Steps

1. Baseline: `cargo run --release -p rafiki-tambua --example bench`
   (expect PASS, note throughput and windows).
2. Edit the bench loop to skip mic samples for the whole run.
3. Rerun. Record: which 9 features go missing, which classes lose
   confidence, whether the bench still passes and why/why not.
4. Restore mic. Repeat for GPS (expect terrain to report unknown,
   displacement frozen).
5. Restore everything. Final rerun must PASS unchanged.

## Expected outputs

- No-mic run: mic.* features absent or stale; acoustic classes drop
  to low confidence or unknown; motion/terrain unaffected.
- No-GPS run: displacement frozen at last value; terrain reports
  unknown (quality gate working as designed, not a bug).
- Restored run: identical to baseline (deterministic seeds).

## Questions

(a) Why does terrain report unknown instead of guessing?
(b) Which single stream, if lost, degrades motion state most, and why?
(c) Where in the code does GPS quality weight downstream confidence?
