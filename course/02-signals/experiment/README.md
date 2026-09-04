# Module 02 experiment: signals

## Objective

Read a live simulator stream the way Teka does, and decide which
sensor to trust least indoors, with numbers.

## Steps

1. `cargo run --release -p rafiki-sae --example simulate`
2. Copy the first three EVENT lines (accel, baro, mic with conf values).
3. In `units.html`, set GPS accuracy to 4, then to 40. Record both
   confidence outputs.
4. Answer in one line each: (a) which sensor has the lowest
   confidence indoors and why, (b) what happens to trust as GPS
   accuracy degrades, (c) why amplitude-only mic data is enough for
   everything downstream.

## Expected outputs

- Simulate output ends in sample counts and a memory line.
- Trust at 4 m is about 0.91; at 40 m about 0.55 (same formula as code).
- (a) GPS: walls kill accuracy, confidence follows it down. (b) Trust
  falls linearly with reported accuracy. (c) Every downstream feature
  uses envelope shape and level, never content.
