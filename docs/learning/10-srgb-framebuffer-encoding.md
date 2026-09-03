# Accelerating sRGB framebuffer encoding

Rendering calculations remain in linear RGB, but the display-ready framebuffer
stores 8-bit sRGB. The exact output transfer for a linear channel `l` is:

```text
s(l) = 12.92 l                         when l <= 0.0031308
s(l) = 1.055 l^(1 / 2.4) - 0.055      otherwise
```

Evaluating the power for every accepted fragment is expensive in WebAssembly.
Apollo 18 instead samples this transfer curve once at 4,097 uniformly spaced
points. A channel is clamped to `[0, 1]`, scaled to one of the 4,096 table
intervals, and linearly interpolated:

```text
p = clamp(l, 0, 1) * 4096
i = min(floor(p), 4095)
t = p - i
s(l) ~= table[i] + t * (table[i + 1] - table[i])
```

The interpolated sRGB value is multiplied by 255 and rounded to the nearest
integer, preserving the framebuffer's existing quantization rule. Shading and
interpolation never enter sRGB space. The table is initialized when the
framebuffer is created, before rasterization can accept a fragment.

## Choosing the table size

Measurements sampled one million linear inputs across `[0, 1]`. Tables with
257, 513, 1,025, and 2,049 samples had worst-case errors of approximately
0.448, 0.145, 0.063, and 0.017 output codes respectively, but each changed at
least one byte in the exact triangle golden. The 4,097-sample table measured a
worst-case error below 0.005 output codes and was the smallest tested table
that preserved all existing goldens. Its single-precision samples occupy about
16 KiB.

The lunar color map is also converted from sRGB bytes to linear RGB once when
the map is constructed. Sampling a fragment therefore performs neither input
nor output power-function evaluation.

## Browser result

The repeatable release test renders the 800x800 level-5 lunar globe for eight
seconds after a two-second warmup. On an Apple M3 Pro running macOS 15.7.7 with
headless Chrome for Testing 151.0.7922.34, the final implementation measured
42.08 FPS, exceeding the required sustained 30 FPS.
