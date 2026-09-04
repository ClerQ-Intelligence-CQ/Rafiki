# Module 08 experiment: the full chain

## Objective

Run all five engines as one pipeline and reproduce every proof in
one sitting.

## Steps

1. `cargo run --release -p rafiki-assembly --example full_pipeline`
2. Record all eight output lines (single, multi, stale, reflect,
   clear, bytes, tracks, throughput).
3. Halve the Tambua convergence window in source, rerun, restore.
4. Rerun to confirm baseline numbers return.

## Expected outputs

- single 0, multi in the hundreds with sensor families, stale true
  then resolved, reflected then cleared, about 5 kilobytes, 30
  tracks flat, five-figure throughput.
- Halved window: co-spike still fires (window still covers it);
  throughput nearly unchanged (window size is not the cost driver).
- Restored: identical (deterministic seeds).

## Questions

(a) Why does halving the window barely move throughput?
(b) Which number would change first if GPS died permanently, and why?
(c) Where is the assembly boundary drawn, and what would violate it?
(Answer: engine crates must stay untouched; any wiring-side logic
beyond orchestration belongs in an engine.)
