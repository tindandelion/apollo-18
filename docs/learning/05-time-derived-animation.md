# Deriving animation from scene time

Animation should describe a pose at a point on a timeline, not a history of frame updates. Apollo 18 therefore supplies explicit **scene time** to each cube render and derives the complete object transformation from that value.

## Rotation as a function of time

The cube keeps its pitch fixed at `-20°`. Its yaw starts at `30°` and completes one revolution every 10 seconds. For scene time `t`, first reduce time into the repeating interval:

```text
loop_time = t mod 10 s
```

Then calculate yaw:

```text
yaw(t) = 30° + 360° × loop_time / 10 s
pitch(t) = -20°
```

Reducing time before calculating the angle keeps the input to the transform bounded. It also makes times separated by a whole 10-second period describe the same pose. As in the static cube stage, yaw is applied before pitch.

## Why frames do not advance the animation

An accumulated animation might update its state after every frame:

```text
yaw = yaw + angular_speed × frame_delta
```

That makes the current pose depend on every earlier update. Dropped frames, different frame rates, or accumulated floating-point error can then change the result.

Instead, Apollo 18 renders the pose directly from explicit scene time. Equal scene times and scene inputs produce equal framebuffers regardless of how many other frames were rendered before them. Native and browser hosts may obtain time differently, but neither host owns the cube's rotation behavior.

## Sampling a native frame sequence

The native cube sequence contains 300 frames at 30 frames per second. Each timestamp is derived independently from its zero-based frame index `n`:

```text
t(n) = n / 30 s, for n = 0, 1, ..., 299
```

The samples cover `0` through `299/30` seconds. The sequence deliberately omits a sample at exactly 10 seconds because it would duplicate the initial pose and introduce a repeated frame at the loop boundary.

Deriving `t(n)` from the index, rather than repeatedly adding `1/30`, avoids cumulative timing drift and makes every numbered frame reproducible in isolation.
