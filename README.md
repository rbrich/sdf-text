sdf-text
========

Render text using Signed Distance Field technique.

**Status: Experimental, no lib API**

Examples:
- `glyph.rs`
  - draw a glyph into texture and render it on a quad
  - `[F1]` shader: alpha-tested  / outlined / direct-linear / direct-nearest
  - `[F2]` texture: SDF / monochrome / freetype-monochrome
- `distance.py`
  - visualize algorithms for finding neareast point on beziér curve
