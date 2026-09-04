# Module 04 experiment: DSP under load

## Objective

Prove bounded cost is load-bearing by breaking it on purpose.

## Steps

1. Run the sustained SPE bench once and record windows, throughput, PASS:
   `cargo run --release -p rafiki-tambua --example bench` (drives the
   full acquisition-to-features chain; SPE numbers print per family).
2. In the engine source, find MAX_LAG (16) and the 64-sample cap in
   `periodicity`. Temporarily raise the cap to 4096 and rerun.
3. Restore both constants. Rerun and confirm PASS again.

## Expected outputs

- Baseline run: PASS, six-figure events/sec.
- Broken run: visibly slower throughput, same PASS (correctness
  holds, cost explodes). This is the O(n^2) failure mode the real
  validation rejected: right answers, wrong cost curve.
- Restored run: numbers back to baseline.

## Questions

(a) Why is a slower-but-correct autocorrelation still a defect here?
(Answer: cost grows with sustained load on a fixed device; a 4GB
no-GPU target cannot absorb unbounded per-event work.)
(b) Where else in the pipeline is cost bounded by constants? (Answer:
windows, lag cap, Goertzel bands, window cap on convergence.)
