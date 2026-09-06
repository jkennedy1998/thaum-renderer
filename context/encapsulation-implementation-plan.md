# Thaum Renderer Encapsulation Implementation Plan

Thaum-renderer already has the contract scaffolding. This plan is for implementation only: bring each existing encapsulation online in an order that matches the current repo contracts, prove the seams in a real Linux window, and avoid drifting into a second invented architecture. Linux is the first live target. Windows comes later only after the real renderer path is stable here.

## Status key
- `[ ]` not done
- `[+]` implemented
- `[#]` tested

## Phase 1 — repo runtime shell
- [+] add a Cargo workspace at `thaum-renderer/`
- [+] add only the minimum Rust crate/module layout needed to host the renderer runtime and one bootable test consumer
- [+] keep code layout mapped to the existing repo ownership shape rather than inventing a parallel runtime tree
- [#] add Linux-first build and run command(s)
- [#] verify the workspace builds cleanly before renderer behavior lands
- [+] add or verify Rust build ignore rules

## Phase 2 — `tools/window-surface/`
- [+] implement reusable window creation helpers for a real graphics window
- [+] implement reusable GPU presentation-surface setup helpers
- [+] implement resize-safe surface reconfiguration helpers
- [+] keep this seam low-level and reusable, not the home of renderer policy
- [#] prove a real Linux window and presentable surface can be created through this seam
- [ ] add pure helper tests only if this seam grows isolated logic worth testing

## Phase 3 — `orchestration/boot/`
- [+] implement the minimal boot seam for standing the renderer up
- [+] accept boot-time window target, asset-root, and hot-reload mode intake here
- [+] keep orchestration boot-only and lean
- [#] prove the renderer can be started through this seam rather than ad hoc wiring
- [ ] add smoke coverage for any pure boot config/default helpers if they appear

## Phase 4 — `tests/` and external consumer proofing
- [+] keep renderer-local `tests/` for proof checks rather than a second app launcher
- [+] use `thaum-renderer-test-user/` as the main manual validation surface
- [+] add one deterministic first scene for visual validation in the external consumer
- [#] use that consumer to prove real-window bring-up on Linux
- [#] prove a first deterministic proof primitive can be drawn in a known place
- [+] use that consumer as the running proof surface for later composition, material, shader, and post-effect checks
- [#] record the canonical local rebuild/run command if repo notes need it

## Phase 5 — `domain/coordinate-space/`
- [+] implement the renderer-owned shared world xyz space
- [+] keep world xyz distinct from cell-group local xyz in the code shape
- [+] make this space traversable and stable enough for later world-space and screen-space shader inputs
- [#] prove known world coordinates land in known visible places
- [#] add pure transform tests if coordinate helpers split out cleanly

## Phase 6 — `domain/camera/` and `tools/camera/`
- [+] implement the canonical camera state over global coordinate space
- [+] implement camera position and focus target xyz
- [+] keep pan, roll, swing, focus-depth, and projection ownership with camera state
- [+] place only reusable calculations like visible-depth-range under `tools/camera/`
- [#] prove the first scene can be viewed reliably with a stable camera
- [#] add pure tests for reusable camera helpers if they emerge

## Phase 7 — `domain/cell-group/`
- [+] implement the runtime cell-group container as the layer-level renderer unit
- [+] implement local coordinate handling inside the group
- [+] implement sparse storage as the real heavy-use storage path
- [+] implement group transform, bounds, and clipping
- [+] preserve six-cardinal facing constraints
- [#] prove a group can hold cells and occupy known world space correctly
- [#] add pure tests for sparse storage and clipping helpers if isolated

## Phase 8 — `domain/composition/`
- [+] implement composition over shared world space
- [+] implement explicit render ordering as composition-owned behavior
- [+] implement exact-xyz overlap behavior where the last rendered cell wins
- [+] avoid hidden fallback ordering
- [#] prove two groups compose correctly in both overlapping and non-overlapping cases
- [#] add pure tests for overlap and pass-order resolution if isolated

## Phase 9 — `domain/cell/`, `domain/cell-weight/`, `domain/cell-color/`, `domain/cell-graphic/`
- [+] implement the atomic runtime cell shape
- [+] implement the canonical four consumed slots: graphic, color, weight, and shader stack state
- [+] keep the cell pixel-like and local to its group rather than letting group or composition semantics leak into it
- [+] implement the canonical weight range `0`, `1`, `2`, `3`
- [+] implement the base flat-color path in `cell-color/`
- [+] keep one shared cell boundary for glyph-backed and sprite-backed graphics
- [ ] choose one first proving graphic path only: `cell-graphic/glyph/` or `cell-graphic/sprite/`
- [+] prove one real cell renders in a known position through a real group and composition path
- [+] add pure tests for any slot-resolution helpers that remain logic-only

## Phase 10 — first proving graphic path
- [ ] if glyph-first, implement `domain/cell-graphic/glyph/` with `glyph-color-space/` rules
- [ ] if glyph-first, preserve black-as-transparent and white-to-medium-light default decode behavior
- [ ] if sprite-first, implement `domain/cell-graphic/sprite/` with `sprite-color-space/` rules
- [ ] if sprite-first, preserve the six channels `A`, `B`, `C`, `A+B`, `B+C`, `C+A`
- [ ] if sprite-first, preserve the four fixed value bands and 24-color canonical palette shape
- [ ] if sprite-first, preserve nearest-color matching with no warning path
- [ ] keep whichever path lands first compatible with the shared `cell-color/` and `cell-materials/` seams
- [ ] avoid pretending a proving simplification is the real sprite contract if sprite-first is chosen

## Phase 11 — `tools/color/`
- [ ] implement fast nearest-color matching helpers
- [ ] keep this seam reusable and low-level rather than embedding sprite policy here
- [ ] use it to support sprite decode and future indexed post-processing work
- [ ] prove nearest-color matching behaves deterministically against a provided collection
- [ ] add pure tests here because this seam is logic-heavy and isolated

## Phase 12 — `domain/data-lanes/` and `workers/breath-fallback-clock/`
- [+] implement renderer-wide data-lane intake
- [+] implement `breath` as the first renderer-standard int lane
- [+] keep renderer consumption simple and app-agnostic
- [+] implement the optional fallback breath worker only for cases where no app-provided breath exists on boot
- [+] preserve the default fallback cadence of `+1` every `.25` seconds
- [+] preserve safe wrap behavior to avoid overflow
- [#] prove breath values reach runtime consumers cleanly
- [#] add tests for fallback tick behavior if isolated cleanly

## Phase 13 — `orchestration/asset-root/` and `domain/atlas-intake/`
- [ ] implement boot-provided asset-root resolution
- [ ] implement lookup for the renderer-facing asset folders expected under that root
- [ ] preserve the boot-time folder expectations for materials, cell sprites, cell shaders, cell effects, and post effects
- [ ] implement atlas intake as its own seam, separate from sprite color-space
- [ ] preserve current atlas forms: `single`, `six-sided`, and `connecting-cardinal`
- [ ] preserve current stacked-weight interpretation for atlas forms that vertically pack weight variants
- [ ] prove one atlas texture and its metadata load successfully from a consumer-chosen root
- [ ] add tests for atlas metadata parse and validation logic

## Phase 14 — `domain/cell-materials/`
- [ ] implement the shared four-band material runtime shape
- [ ] implement the built-in `gray-scale` material for early validation
- [ ] preserve shared material resolution for both glyph and sprite paths
- [ ] keep renderer-owned test materials separate from consumer-authored external material libraries
- [ ] prove one known materialized render result in the test scene
- [ ] add tests for material parse, defaulting, and lookup helpers if isolated

## Phase 15 — `domain/cell-shader/`, `domain/cell-adjacency/`, `domain/cell-blur/`, `domain/cell-warble/`
- [+] implement the per-cell shader stack runtime path
- [+] preserve `0` as pass/nothing when used in shader-stack state
- [+] implement the first built-in proving shader as `weight-sin`
- [+] preserve `weight-sin` as breath-driven relative weight modulation over world xyz
- [+] clamp shader output back into the valid cell-weight range
- [ ] implement adjacency as optional rendering-only input data
- [ ] only pay adjacency cost when something actually consumes it
- [ ] keep blur and warble as shader-driven seams without over-expanding them before proofing
- [#] prove breath visibly modulates output in the test scene
- [#] add tests for pure shader registry or stack-resolution helpers if isolated

## Phase 16 — `domain/post-effects/`
- [ ] implement the ordered post-effects stack over the fully composed image
- [ ] implement `index-color-clamp/` as the first proving post effect
- [ ] keep `index-color-clamp` last in the stack
- [ ] keep post effects image-wide and after composition, not per-cell
- [ ] prove the final image changes through post processing rather than cell-local logic
- [ ] add tests for pure post-effect config/default helpers if they appear

## Phase 17 — `workers/asset-hot-reload/`
- [ ] implement optional watching of the chosen renderer asset root
- [ ] keep the watcher scoped to renderer-relevant assets only
- [ ] support atlas, material, shader, cell-effect, and post-effect reload signaling as those asset families become real
- [ ] keep hot reload boot-configurable and cleanly disableable
- [ ] prove hot reload can be enabled and disabled cleanly
- [ ] add tests for debounce or change-state helpers if isolated

## Phase 18 — Linux-first full proof
- [ ] prove the renderer boots on Linux through the real orchestration seam
- [ ] prove a real window opens and resizes correctly
- [ ] prove coordinate-space, camera, cell-group, and composition all work together
- [ ] prove one real graphic path works correctly
- [ ] prove one material, one shader, and one post effect all work together in the same scene
- [ ] prove asset-root loading works from a consumer-chosen path
- [ ] prove the test scene is sufficient as the main manual validation seam
- [ ] note Linux-specific quirks before later Windows follow-over

## Implementation notes
- landed slices:
  - Cargo workspace and crate shell mapped onto repo ownership shape
  - `tools/window-surface/` now opens a real `winit` window, creates a `wgpu` presentation surface, reconfigures on resize, clears to a deterministic dark color every frame, and can draw submitted quads
  - `orchestration/boot/` launches the window seam through a lean boot config and can project renderer-owned cell/composition scenes into surface quads
  - `domain/` now carries a first explicit runtime slot shape for `cell-weight`, `cell-color`, `cell-materials`, glyph-backed `cell-graphic`, and shader-driven `cell-texture`
  - the first glyph proof path now rasterizes simple binary glyph masks into per-pixel subquads, with transparent background behavior, fixed-amplitude texture shimmer support, and default medium-light material resolution for glyph pixels
  - `thaum-renderer-test-user/` remains the canonical manual validation launcher and now submits flat-color and grayscale-material glyph proof cells through composition instead of the old ad hoc proof primitive
- tests run:
  - `source ~/.cargo/env && cargo check --workspace`
  - `source ~/.cargo/env && cargo test --workspace`
  - `source ~/.cargo/env && cd thaum-renderer-test-user && cargo build --release`
- dirty/uncommitted follow-up still expected before git:
  - replace the inline proof glyph masks with a fuller glyph-source intake path aligned with thaum mono expectations
  - wire live glyph decode against the staged repo-local asset root
  - wire first sprite intake proof against the staged repo-local asset root
  - explicit smoke tests for pure boot/window helper selection logic if worthwhile
  - GPU adapter fallback notes for restricted Linux environments
- current first proving graphic path:
  - glyph-backed proof cells now rasterize visible glyph pixels as subquads resolved through real cell-color/material slot semantics
- Linux runtime proof notes:
  - on JOBO, the external test-user app boots through the real event loop and stays alive
  - EGL warned about `/dev/dri/renderD128` and `/dev/dri/card1` permission denial in this environment, so adapter fallback behavior should be expected during development runs here
- staged asset-root proof notes:
  - repo-local proving assets now live under `orchestration/renderer-assets/`
  - `orchestration/renderer-assets/glyph-fonts/thaum-mono/` currently stages the extracted Thaum Mono weight files
  - `orchestration/renderer-assets/cell-sprites/proofs/grass.png` currently stages the first sprite intake proof file
- Windows follow-over notes:
  - defer until Linux path has at least one asset-backed or atlas-backed cell graphic path beyond the current inline proof glyph masks
