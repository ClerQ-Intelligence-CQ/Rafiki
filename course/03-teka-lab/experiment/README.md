# Module 03 experiment: acquisition

## Objective

Measure what the acquisition engine actually delivers against what a
real sensor stream needs.

## Steps

1. `cargo run --release -p rafiki-sae --example simulate` (record the
   events/sec line and the DUTY line).
2. `cargo test -p rafiki-sae` (expect 3 passed).
3. In `duty.html`, press Still 10 times, then Walking 5 times, and
   copy the resulting log.

## Expected outputs

- Simulator delivers millions of events/sec in-process (far above
  the 100 events/sec a real 20 Hz five-sensor stream produces).
- Duty sits at 500 ms through still samples, drops to 50 ms on
  motion, first sample always active.
- Tests: 3 passed.

## Questions

(a) How much headroom does the engine have over real time, in multiples?
(b) Why does the first sample always count as active?
(c) What would break if confidence were a fixed 0.95 for every sample?
(Answer: a bad sensor could certify the batch; the worst-stream rule exists to stop exactly that.)
