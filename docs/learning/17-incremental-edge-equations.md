# Incremental edge equations

Barycentric interpolation tells us *what* to calculate for a covered sample. It does not require us to recalculate the whole expression independently at every pixel. This distinction matters when the same small calculation runs millions of times per frame.

## The original approach

For a directed edge from `A` to `B`, Apollo 18 evaluates a pixel-center sample `P` with:

```text
E(A, B, P) = (B.x - A.x)(P.y - A.y) - (B.y - A.y)(P.x - A.x)
```

A triangle has three such equations. Their values decide whether the sample is covered, including the top-left rule for exact zeros. The same values become barycentric weights after division by the signed triangle area:

```text
w0 = E0 / area
w1 = E1 / area
w2 = E2 / area
```

This direct approach is an excellent first implementation. Every loop iteration constructs `P`, subtracts the edge start, and evaluates three complete edge equations. The relationship between coverage and interpolation is easy to inspect.

## Where repeated work costs time

The level-5 lunar globe contains 8,192 triangles. Each triangle traverses a framebuffer-bounded rectangle of candidate pixel centers. Many candidates fail coverage, but all of them still need three edge values before the software renderer can know that.

In the original inner loop, neighboring samples repeat most of the same arithmetic. When `P.x` changes from `x + 0.5` to `x + 1.5`, the edge direction and `P.y` have not changed. Reconstructing the complete expression hides that useful regularity.

Covered samples also used three divisions to normalize the edge values. Division is generally more expensive than multiplication in WebAssembly. Since every weight has the same denominator, calculating the reciprocal once per triangle can replace those divisions with multiplications.

These are small costs individually. They matter because they sit inside the candidate-fragment loop. Performance work should therefore consider both the cost of one operation and how often it runs.

## An edge equation is affine

Let the edge direction be:

```text
dx = B.x - A.x
dy = B.y - A.y
```

Expanding the edge equation shows that it is affine in the sample coordinates:

```text
E(x, y) = dx(y - A.y) - dy(x - A.x)
```

Moving one pixel to the right changes only `x`:

```text
E(x + 1, y) = E(x, y) - dy
```

Moving one row down in the top-left framebuffer changes only `y`:

```text
E(x, y + 1) = E(x, y) + dx
```

The increments `-dy` and `dx` are constant for the entire edge. The Ticket 34 experiment evaluated all three edge equations once at the bounded traversal origin, added each edge's X increment across a row, and added each edge's Y increment to the saved row-start values between rows.

Conceptually, the experimental loop changed from this:

```text
for every candidate (x, y):
    construct its pixel center
    evaluate E0(x, y), E1(x, y), E2(x, y)
```

to this:

```text
row_values = evaluate all edges at the first pixel center
for every row:
    values = row_values
    for every candidate in the row:
        use values
        values += x_steps
    row_values += y_steps
```

Coverage still used the three edge values, and interpolation still used those exact same values. The experiment changed how they were produced, not what they meant.

## Normalize once per triangle

The triangle area is constant, so its reciprocal is constant too:

```text
inverse_area = 1 / area
wi = Ei * inverse_area
```

This would move one division outside candidate traversal and replace three divisions per covered sample with multiplication. Floating-point multiplication by a reciprocal is not guaranteed to round identically to division, so this is not merely an algebra exercise: exact small-scene goldens and realistic lunar goldens must prove that the observable output remains acceptable.

## Optimize only with evidence

An optimization can make source code look faster while producing no measurable benefit. Apollo 18 compares warmed release-browser runs of the same 1152×1152 workload and keeps visual behavior protected by focused rasterizer tests and golden renders.

For Ticket 34, three baseline runs had median complete-frame times of 42.45, 42.80, and 42.50 ms. Incremental stepping alone measured 42.70, 42.95, and 43.95 ms, so no isolated improvement was claimed. Reciprocal normalization paired with direct edge evaluation regressed to 45.55, 45.60, and 45.55 ms. Combining both changes measured 41.60, 41.40, and 41.60 ms. Exact triangle and cube goldens and tolerance-checked lunar goldens remained unchanged. This is a useful reminder that compiler output and surrounding loop structure matter: timings for isolated source edits do not necessarily add together.

The best variant reduced median whole-frame time by only 0.9 ms, or 2.1%, and throughput remained about 23 FPS against the 30 FPS target. That gain did not justify carrying extra floating-point accumulation risk and loop complexity, so all arithmetic changes were reverted. Apollo 18 continues to evaluate edge equations directly and divide edge values by area. The horizontal shared-edge regression test remains because it strengthens coverage of the unchanged top-left rule.

Tighter per-scanline traversal bounds were not attempted. They could skip more outside candidates, but they add more edge-intersection and rounding behavior near the top-left coverage boundary. A future residual profile should demonstrate that this risk is necessary before revisiting the traversal shape.

## Correctness risks to remember

Incremental floating-point addition can accumulate rounding differently from direct evaluation. A tiny difference is most dangerous when an edge value is exactly or nearly zero, because it may change shared-edge ownership. A changed barycentric weight can also affect depth ties, map lookup, or final color.

That is why the experiment was checked against:

- both windings and degenerate triangles;
- fully and partially off-screen triangles;
- diagonal and horizontal shared edges in both draw orders;
- strict depth ties;
- exact triangle and cube golden framebuffers;
- realistic lunar goldens and deterministic native output.

Fast arithmetic is useful only while those rendering rules remain intact.
