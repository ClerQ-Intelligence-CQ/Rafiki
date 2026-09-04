# Module 02: Signals 101 (30 minutes)

Goal: know what each sensor measures, in what units, and what noise
looks like. Everything downstream assumes this.

## 1. The five streams

- **Accelerometer:** acceleration along three axes, meters per second
  squared. At rest on a table it reads about 9.81 on the vertical
  axis: that is gravity, not motion. Motion is change on top of it.
- **GPS:** latitude and longitude in degrees, plus accuracy in
  meters. Phone GPS wanders several meters even standing still; the
  accuracy number says how much to trust each reading.
- **Barometer:** air pressure in hectopascals, plus temperature in
  Celsius. Pressure converts to approximate altitude. Weather moves
  it slowly; stairs and elevators move it fast.
- **Microphone:** amplitude envelope in decibels plus dominant pitch
  in hertz. Envelope only: how loud, never what was said. Content is
  never recorded, never stored. This is a hard rule, not a setting.
- **Screen state:** on or off, plus brightness percent. The cheapest,
  most honest behavior signal available.

## 2. Noise, plainly defined

**Noise** is variation that carries no information about what you
want to know: GPS jitter while standing still, ±0.03 wobble on a
resting accelerometer, a few dB of mic flutter in a quiet room. Every
engine downstream either filters noise out (windows, averages) or
measures against it (variance, thresholds relative to observed
spread). Nothing here uses a fixed number from a textbook; noise is
always estimated from the live stream itself.

## 3. Sampling in one paragraph

Sampling means reading a sensor on a schedule. Faster sampling sees
more detail and burns more battery. Teka samples fast (50 ms) when
the device is moving and slow (500 ms) when quiet, switching on
measured activity. You will run this yourself in Module 03.

## 4. Try it

Open `units.html`: convert GPS accuracy to trust levels, pressure to
altitude, and watch what noise does to a resting accelerometer trace
(computed live in the page, no server).

Experiment `experiment-m02-signals.zip`: run the simulator, read the
confidence column per sensor, and answer three questions in its
README about which sensor you would trust least indoors.

Next: [Module 03](../03-teka-lab/guide.md).
