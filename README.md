sdf-text
========

Render text using Signed Distance Field technique.

**Status: Experimental, no lib API**

Examples:
- `glyph.rs`
  - draw a glyph into texture and render it on a quad
  - `[F1]` filtering: bilinear / nearest
  - `[F2]` shader: alpha-tested  / outlined / straight
  - `[F3]` texture: SDF / monochrome
- `distance.py`
  - visualize algorithms for finding neareast point on beziér curve
