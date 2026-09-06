# /home/j/Repos/thaum-renderer/domain/modules/individuals/camera-perspective

## purpose
Renderer-owned camera perspective panel: direct wheel/click tuning of the shared [`PerspectiveProfile`] and [`ParallaxProfile`] knobs, the camera's render depth, and the camera's rendered-layer count, so any host that opts into renderer modules can hand its users live perspective control.

## owns
- `camera_perspective_module.rs`
  - the `CameraPerspectiveModule` panel: rows `scale`, `position`, `floor`, `parallax`, `str`, `depth`, `layers`, plus the reserved bottom hint row
  - wheel-over-a-row fine-tune / click cycles presets semantics (parallax toggle flips, depth click re-centers at 0, layers click cycles layer-count presets)
  - the `CameraDepthLink` host ↔ panel bridge shape (pending wheel steps in, host-published live depth out)
  - the `CameraLayersLink` host ↔ panel bridge shape for the rendered-layer count, plus the `MAX_VISIBLE_PLANE_RADIUS` cap and layer-count presets
  - panel chrome/gizmo behavior (move/close/resize/seamless) and module ui-state persistence

## does not own
- the `Camera` itself — the host owns it, drains `CameraDepthLink` pending steps into `Camera::pan_focus_depth`, drains `CameraLayersLink` pending steps into `Camera::visible_plane_radius` (clamped to `0..=MAX_VISIBLE_PLANE_RADIUS`), and publishes `Camera::focus_depth` / `Camera::visible_plane_radius` back
- `PerspectiveProfile` / `ParallaxProfile` math — owned by `domain/camera/perspective/` and `domain/camera/parallax/`
- wheel behavior off the panel rows — falls through to the host's viewport camera handling

## children-encapsulations
- none

## dependencies
- `domain/modules/shared/panel-chrome/` — panel chrome and gizmo bar
- `domain/modules/shared/property-rows/` — row layout/hit-test convention
- `domain/camera/perspective/` — `PerspectiveProfile` knob values and clamps
- `domain/camera/parallax/` — `ParallaxProfile` toggle/strength
- `domain/modules/module.rs` — `Module` trait surface

## exposed interfaces
### CameraPerspectiveModule::new — build the panel
send: { id: string, rect: ModuleRect, profile: Rc<RefCell<PerspectiveProfile>>, parallax: Rc<RefCell<ParallaxProfile>>, depth: Rc<CameraDepthLink>, layers: Rc<CameraLayersLink> }
returns: CameraPerspectiveModule
effects: none

### profile / parallax_profile / depth_link — shared handles for host binding
returns: Rc handles the host syncs into its camera (perspective + parallax each frame; depth link drained/published each frame)
effects: none

### CameraDepthLink — render depth bridge
- `scroll(steps)` — panel-side: accumulate one signed step per wheel notch (matches the canvas depth scroll's per-event step counting)
- `drain_pending()` — host-side: take and clear the accumulated steps
- `set_current(depth)` / `current()` — host publishes the live `Camera::focus_depth`; the depth row displays it

### CameraLayersLink — rendered-layer-count bridge
- `scroll(steps)` — panel-side: accumulate one step per wheel notch (one rendered layer per side of the focus plane)
- `drain_pending()` — host-side: take and clear the accumulated steps
- `set_current(radius)` / `current()` — host publishes the live `Camera::visible_plane_radius`; the layers row displays it
- `MAX_VISIBLE_PLANE_RADIUS` = 512 is the cap the host clamps the drained radius to; click cycles the presets `0, 1, 2, 4, 8, 12, 16, 24, 32, 64, 128, 256, 512`

### Module trait surface
- `on_wheel` consumes only wheel events over panel rows (depth row included); everything else reports unhandled so viewport camera behavior keeps working
- `on_pointer_event` handles gizmos first, then row clicks

## interface consumers
- none declared yet (host wiring: `thaum-painter/orchestration/entrypoint/`)

## artifacts
- none

## tests
- `camera_perspective_module`
  - light
  - validates row hit-testing, wheel fine-tune/clamp, preset cycling, parallax toggle/strength rows, depth row scroll/reset/display through the link, layers row scroll/preset-cycle/display through its link, fall-through off rows, gizmo behavior, ui-state persistence, and draw anchoring

## data
- none

## notes
- Rows stack downward from the top content row; draw output is module-local, events are screen-space (shared module convention).
- Depth edits intentionally route through the host (link, not direct camera mutation) so keys and canvas wheel-scroll stay the only other writers and the camera keeps a single owner.
- Depth follows the swing via `Camera::focus_depth`, exactly like `pan_focus_depth`, and persists with the camera's focus target in host session state.
- The layers row edits `Camera::visible_plane_radius` — the rendered-layer count on each side of the focus plane — which was previously host-fixed with no UI; it already persists with the camera in the renderer's ui-session-state, so user changes survive restarts with no new persistence fields.
- The `depth` row stays the focus-plane center; the `layers` row is the window size. Two separate knobs on purpose.
