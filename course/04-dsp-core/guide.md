# Module 04: DSP core (40 minutes)

Goal: understand every mathematical tool Chuja uses, why each exists,
and why the naive versions were rejected. This is the longest theory
module; Module 05 applies it.

## 1. Windows

A window is a recent slice of the stream. Short windows (32 samples,
about 1.6 seconds at 20 Hz) catch transients like knocks. Long
windows (256 samples) track states like sustained motion. Everything
downstream reads windows, never raw history, so memory stays flat.

## 2. Mean, variance, zero crossings

Mean: the middle of the window. Variance: how spread out it is.
Zero-crossing rate: how often the signal crosses its own mean,
which separates rhythmic motion (high) from drift (low). All O(n),
single pass, no history stored.

## 3. Frequency without a full FFT

Dominant frequency band comes from Goertzel probes at three
normalized frequencies, O(n) per band. Walking shows up in the
middle band (gait rhythm); stillness sits in the low band; fast
irregular motion in the high band. A full FFT would work but costs
more than this problem needs.

## 4. Autocorrelation, bounded

Autocorrelation asks: does the signal repeat itself after some lag?
The honest version computes only lags 1 to 16 over at most 64
samples: O(16*64) per call, bounded forever. The naive full O(n^2)
version hung the first validation under sustained load and was
rejected. Bounded cost under sustained load is a design rule here,
not an optimization.

## 5. Adaptive gates, never fixed numbers

Transient detection fires against noise floor plus 3 standard
deviations, where the floor itself is a slow moving average of the
live stream. A quiet bedroom and a loud street get different gates
automatically. Any fixed threshold from a textbook would be wrong
half the time; the gate must learn the room it is in.

## 6. Gravity separation and GPS smoothing

Accelerometers measure gravity plus motion mixed together. A slow
exponential average tracks gravity (orientation: pitch and roll)
while the remainder carries motion. GPS jitters meters per reading,
so positions smooth over a short moving average before any rate is
computed from them.

## Try it

`features.html` runs a live autocorrelation and Goertzel demo in the
page: feed it still, walking, and driving traces and watch which
band wins and how periodicity tracks rhythm.

Experiment `experiment-m04-dsp.zip`: break one thing deliberately.
Halve the autocorrelation lag cap and rerun the Chuja bench; record
what changes in the periodicity numbers and explain why boundedness
is load-bearing, not cosmetic.

Next: [Module 05](../05-chuja-lab/guide.md).
