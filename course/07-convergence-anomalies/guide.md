# Module 07: convergence and anomalies (30 minutes)

Goal: understand why one loud signal is noise and several together
are signal, and how the bar scales itself.

## 1. Deviation, scored honestly

Each live value scores z = |value - baseline mean| / baseline std,
with confidence scaled by that baseline's stability. Low stability
does not suppress the score; it flags it low-confidence. The system
stays transparent about what it sees even when unsure. That is a
design choice: alarmist systems hide uncertainty, this one reports it.

## 2. Why one signal is never enough

A microphone spike alone could be a dropped pan. An accelerometer
spike alone could be a pothole. Either, with friends from other
sensor families at the same time, becomes evidence. Convergence
means deviations co-occurring across at least two independent
sensor families inside one time window. Classifications and meta
features are scored and shown but never vote, because they derive
from the same streams and would let one loud sensor plus its own
echoes pass as convergence.

## 3. N scales itself

Required deviating features = max(2, trustworthy/8), where
trustworthy counts baselines at high stability. Ten trustworthy
features demand 2; thirty demand 4. As evidence accumulates the bar
rises with it. No constant, no schedule, no population number.

## 4. The window and the floor

The convergence window is capped in entries and evicted by time, so
memory stays flat forever. A scale-relative epsilon floor kills
zero-variance explosions (microscopic float dust scoring z = 1000).
Both rules are documented in code as numerical hygiene, same for
every feature.

## 5. Try it

The two canonical proofs, which you will now run yourself:

- Single mic spike inside steady walking: 0 anomalies. Silence.
- Mic plus accel plus GPS co-spike: fires, contributors exactly the
  sensor families that moved.

Experiment `experiment-m07-spikes.zip`: run the Tambua bench, verify
both numbers, then make the single spike ten times longer and
rerun. Record whether silence holds and explain why duration does
not matter (families, not counts, decide).

Next: [Module 08](../08-twin-assembly/guide.md).
