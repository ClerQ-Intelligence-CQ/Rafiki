# Module 03: Teka lab, acquisition in practice (30 minutes)

Goal: run the acquisition engine, understand confidence, gating, and
duty cycling, and measure throughput yourself.

## 1. What Teka does, precisely

Each sensor sample becomes a typed event with a timestamp and a
confidence score, then passes an activity gate that decides whether
the device is moving. Moving shortens the sampling interval to 50 ms;
stillness stretches it to 500 ms, floored at 10 ms. The event bus (a
plain in-memory list in this public fork) carries events downstream.
Raw samples are never written anywhere.

## 2. Confidence, worked

GPS at 4 m accuracy: 1 - 4/100 = 0.96, clamped to 0.95. GPS at
90 m: 1 - 90/100 = 0.10. Microphone at 32 dB envelope:
0.5 + (32/80)*0.45 = 0.68. Resting accelerometer (magnitude within
a hair of 9.81): near 0.95. Barometer inside 950-1050 hPa: 0.9,
outside: 0.6. Screen state: 0.9, boolean by nature. The batch score
takes the worst stream, so one bad sensor cannot certify the batch.

## 3. The gate and the duty cycle

First sample of each stream always counts as active (nothing to
compare against yet). After that, a sample counts as active only if
it moved more than 0.05 from the last one, in that stream's own
units. Active samples flip the interval to 50 ms; quiet stretches it
back to 500 ms. Watch it happen in the simulate output: the DUTY
line drops on quiet input.

## 4. Try it

```bash
cargo run --release -p rafiki-sae --example simulate
```

Then open the bench behind it: `engines/sae` unit tests cover the
still-to-active transition, confidence ordering, and bus delivery
(`cargo test -p rafiki-sae`).

Experiment `experiment-m03-teka.zip`: run the simulator, record
throughput and the duty line, then answer how many events per second
a real 20 Hz five-sensor stream needs versus what the engine
delivers, and where the headroom goes.

Next: [Module 04](../04-dsp-core/guide.md).
