# Module 06: baselines and the cold start (40 minutes)

Goal: understand Welford statistics, decay weighting, and the
stability formula, then watch a baseline chase a moving world.

## 1. The problem with averages

A plain running average treats January like this morning. For a
living person that is wrong: routines drift, seasons change, bodies
change. The baseline must track drift, which means old observations
must weigh less than new ones, forever, without storing history.

## 2. Welford in plain words

Keep three numbers per feature: how many effective samples, the
current mean, and a helper for variance. Each new sample nudges the
mean toward itself and updates the helper in one pass. No history
stored, constant memory, numerically stable. This is Welford's
algorithm, classical statistics, no training involved.

## 3. Decay: forgetting on purpose

Before absorbing each sample, shrink the accumulated weight by
(1 - 1/500). Fresh samples always count fully; six-month-old
samples have faded to nearly nothing. The effective count caps
around 500, so the baseline stays nimble for life instead of
freezing into its youth.

## 4. The stability formula, derived

Standard error of the mean: SE = sqrt(variance / n). It shrinks as
evidence grows, so it directly measures how much the mean can still
move. Stability S = n / (n + 50 * (1 + CV)), where CV is the
coefficient of variation (std divided by |mean|). Why this shape:
more evidence raises S; low natural spread raises it faster at
equal evidence; it lives in [0, 1) with no cutoff anywhere. A
metronome-regular feature earns trust in hundreds of samples; a
wild one takes thousands. Same formula, different pace, on each
feature's own terms.

## 5. Cold start, solved without a calendar

There is no "ready after N days" anywhere. Downstream simply
weights by S continuously: low-stability baselines whisper, high
ones speak. Day one works (quietly); day thirty works louder. The
open research question from the original brief is answered by this
formula plus the proof below, not by a policy.

## 6. Try it

`stability.html`: drag sample count and spread, watch S climb fast
for tight features and slow for wild ones. Then run the shift lab.

Experiment `experiment-m06-baseline.zip`: move a distribution
mid-run (44 to 70 dB in the bench) and watch the baseline follow
(42.65 to 65.32 measured). Then halve the decay constant and
rerun: record how much faster it tracks and what it costs in
steadiness.

Next: [Module 07](../07-convergence-anomalies/guide.md).
