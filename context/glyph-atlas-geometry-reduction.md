# Glyph atlas geometry reduction (design note)

Status: agreed direction (J, 2026 session), phase A not yet implemented.

## Problem

`raster_to_surface_quads` / `sprite_raster_to_surface_quads` (`orchestration/boot/src/effect_quads.rs`)
expand every glyph/sprite cell into **one quad per lit 12×16 texel**, plus an
invisible texel ring when texture/warble are active. Idle painter scenes carry
~120k–174k quads (~731k–1.05M vertices, ~50–75 MB vertex writes per frame)
that are rebuilt and re-uploaded every frame.

## J's design truths (source of truth)

- Primary target is Windows; also macOS, maybe Android. The Linux/llvmpipe dev
  box is a test environment, not the optimization target. Optimizations must be
  generic GPU-work reduction, not machine-specific tuning.
- The texture/warble **bus seam is the landed, valued design**: author texture
  and warble codes per cell on the bus, apply them in **one post pass**, with
  **no extra geometry**. Earlier texture attempts that added planes/geometry
  were rejected for compute cost.
- Requirements: warble + texture keep working, the typegrid (12×16 cell tile
  shape) stays consistent, and performance must be decent across machines.

## The geometry target

**One quad per cell.** A glyph cell emits a single full-cell quad (covering the
whole 12×16 cell rect, not just lit texels).

The quad-pass fragment shader samples the glyph atlas (alpha coverage) and:

- writes color with alpha = coverage (lit texels only)
- **always writes bus / meta / warble_uv data across the whole cell rect**,
  including unlit texels (alpha = 0)

This preserves the post-pass effects with **zero extra geometry**:

- texture + warble post effects displace a sample and re-read the rendered
  surface (color/bus/warble targets). Today that requires invisible ring quads
  so displaced samples find neighboring data. With whole-cell rects, the bus
  and aux field is continuous across every covered cell, so displaced samples
  (max reach 5.8 texels < cell size) find data without rings.
- `warble_uv_corners_for_texel` is linear in position; interpolating across the
  full cell reproduces the exact same per-fragment values.
- `local_uv` interpolates 0..1 across the cell exactly as texel corners did.
- `texel_uv_size` stays a per-quad constant (cell size / tile dims).

## Phases

1. **A — glyph atlas**: upload glyph tiles (char, weight) once to a GPU atlas
   (all tiles share the 12×16 shape → uniform grid). Boot emits one quad per
   glyph cell carrying the tile's atlas origin. Ring generation removed for
   glyphs. `GlyphFontSet` stays the CPU raster source.
2. **B — sprites**: sprites already have real PNG atlas textures
   (`SpriteAtlasSet`); sample them directly instead of re-rasterizing into
   texel quads. Same whole-cell pattern.
3. **C — cleanup**: delete `raster_to_surface_quads` /
   `sprite_raster_to_surface_quads` per-texel paths and
   `push_texture_buffer_ring`; rework boot tests that assert texel-quad
   semantics (12×16 aspect, weight sin, flat color, debug post effects) to
   whole-cell + atlas semantics.

## Perf expectation

Scene quads drop from ~120k+ to ~2–3k (glyph cells only). Scene build and
upload fall by the same order; vertex writes become negligible. Post-pass cost
is unchanged (same one pass, same sampling). Fragment coverage work moves to
the GPU where per-quad branching replaces per-quad geometry.
