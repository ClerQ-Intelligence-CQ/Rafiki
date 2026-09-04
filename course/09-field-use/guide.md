# Module 09: field use, where this works and its limits (25 minutes)

Goal: know exactly where this system fits, what it costs to run,
what it promises about privacy, and where it breaks.

## 1. Where it fits

Anywhere a small device watches slow-moving human context with no
reliable network: rural clinics tracking rest and activity patterns,
farm safety check-ins, elder care without cameras, off-grid personal
baselines. Common thread: the question is "does today look like
this person's normal days," asked locally, answered locally.

## 2. What it costs

Memory: kilobytes of state (twin about 5 KB, per-feature scalars,
capped windows). CPU: tens of thousands of events per second on an
ordinary laptop CPU, against a real need of about 100 per second.
Battery: duty cycling drops sampling 10x on quiet. No GPU, no
network, no subscriptions.

## 3. Privacy posture, stated plainly

Raw sensor data never persists: accelerometer traces, GPS points,
and mic levels live in rolling windows and leave no disk trace.
Microphone content is never captured (amplitude envelope only).
What persists is derived state: means, variances, counts, flags.
A stolen device yields a 5 KB summary, not a life recording.

## 4. Honest limits

- Needs days of data before baselines earn trust (cold start is
  graceful, not instant).
- Knows nothing about causes, only deviations (a fever and a flu
  look identical to it).
- GPS-denied indoor life weakens terrain and displacement features
  by design (quality gates say so openly).
- Deterministic means repeatable, not means correct: garbage sensors
  produce garbage baselines, confidently. Validate the hardware
  first.

## 5. Failure-mode table (memorize this)

| Failure | Symptom | Detection |
|---|---|---|
| Dead sensor | features freeze | staleness flags |
| Drifting sensor | slow baseline chase | stability drops |
| Single loud event | one family deviates | silence (by design) |
| Sustained multi-stream change | convergence | anomaly fires |
| Full outage | everything ages | completeness collapses |

Experiment `experiment-m09-field.zip`: fill the deployment
checklist (power, storage, privacy, cold-start plan) for one real
place you know, and name which row of the table would fire first
there.

Next: [Module 10](../10-capstone/guide.md).
