# /home/j/Repos/thaum-renderer/tools/window-surface

## purpose
Own reusable helper seams for creating and managing the real graphics window and GPU presentation surface consumed by thaum-renderer.

## owns
- cross-platform renderer window-surface helper organization
- reusable window creation and presentation-surface setup helpers
- low-level presentation concerns shared by renderer runtime paths
- raw OS window input capture (keyboard, cursor position, mouse clicks), surfaced per-frame as `WindowSurfaceInput` in clip-space coordinates matching rendered quad placement

## does not own
- renderer design truth
- app-specific window chrome or UX policy
- post-effect semantics
- cell composition behavior
- what a click or keypress means to a program (module hit-testing, action binding) — see `domain/modules/` and `domain/controls/`
- screen-to-world/cell coordinate remap — see `domain/camera/screen-world-remap/`

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/tools/`

## exposed interfaces
- window-surface helpers
  - describes the reusable helper seam for preparing a real window and GPU-backed presentation target for renderer output

## interface consumers
- future renderer runtime surfaces
- future test scenes

## artifacts
- none

## tests
- none

## data
- none

## notes
- the renderer should present to a real window with real graphics rather than treating png output or terminal output as the primary target
- the preferred implementation direction is a GPU-first cross-platform surface suitable for Linux, Windows, Apple platforms, and Android-class targets
- in Rust, `wgpu` plus `winit` is the current best-fit direction for the primary implementation path unless later research proves a better cross-platform fit
- `WindowSurfaceInput.cursor_position`/`just_clicked` are reported in the same clip-space (-1..1, y up) that `SurfaceQuad::center` renders into, so a consumer can convert them to world/cell coordinates with the same math it already uses for rendering (see `domain/camera/screen-world-remap/`), rather than this crate guessing at cell semantics
- `WindowSurfaceInput.pointer_down` persists across frames for as long as the primary button stays held (unlike `just_clicked`, which only fires the frame the press happens); a consumer combines it with `cursor_position` each frame to drive a drag session, such as `domain/modules/shared/module-gizmos/`'s move/resize gizmos, via `domain/modules/`'s pointer-capture dispatch
- dev machine (jobo) hardware truth from J: Strix Halo, 128 GB unified memory — CPU and GPU share one physical memory pool, so vertex/texture uploads are memcpys into shared GTT, not PCIe transfers, and process RSS is not a meaningful perf signal on this machine
- renderer perf measurement truth on jobo: `:10` is the xrdp software-rendering display (wgpu falls back to lavapipe; GPU sysfs reads 0), while `:0` is the real AMDGPU seat with DRI3 — perf-log numbers are only hardware-truthful when the renderer runs on `:0`
