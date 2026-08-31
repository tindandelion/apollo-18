# Use linear RGB for rendering

Represent colors outside the shared renderer as sRGB, but decode them to linear RGB before interpolation, filtering, or lighting. Encode final linear colors back to sRGB when the shared renderer writes its 8-bit RGBA framebuffer, so native PNG and web presentation consume identical display-ready bytes without duplicating color conversion in either host. This adds explicit conversion and makes simple interpolation less obvious, but keeps color calculations physically coherent from the first interpolated triangle onward.
