# /home/j/Repos/thaum-renderer/domain/modules/individuals/camera-perspective

## purpose
Renderer-owned camera perspective panel: direct wheel/click tuning of the shared [`PerspectiveProfile`] and [`ParallaxProfile`] knobs, the camera's render depth, and the camera's rendered-layer count, so any host that opts into renderer modules can hand its users live perspective control.

## owns
- `camera_perspective_module.rs`
  - the `CameraPerspectiveModule` panel: rows `scale`, `position`, `floor`, `parallax`, `str`, `depth`, `layers`, `zoom`, plus the reserved bottom hint row
  - the standard responsive color ladder on the rows (J 2026-09-12): labels Medium, values Bright at rest and Vivid when the row is engaged (parallax on) or hovered — the panel pins nothing at a static role, so it responds to the live `UiPalette` and to pointer state
  - wheel-over-a-row fine-tune / click cycles presets semantics (parallax toggle flips, depth click re-centers at 0, layers click cycles layer-count presets, zoom click snaps the next zoom preset)
  - the `CameraDepthLink` host ↔ panel bridge shape (pending wheel steps in, host-published live depth out)
  - the `CameraLayersLink` host ↔ panel bridge shape for the rendered-layer count, plus the `MAX_VISIBLE_PLANE_RADIUS` cap and layer-count presets
  - the `CameraZoomLink` host ↔ panel bridge for the camera's zoom (J 2026-09-12): the zoom row sits between `layers` and `quality` and drives the same `Camera::zoom` the − and + keys drive — wheel accumulates one multiplicative step per notch, click queues an absolute preset snap that outranks queued steps for that frame
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
send: { id: string, rect: ModuleRect, profile: Rc<RefCell<PerspectiveProfile>>, parallax: Rc<RefCell<ParallaxProfile>>, depth: Rc<CameraDepthLink>, layers: Rc<CameraLayersLink>, zoom: Rc<CameraZoomLink> }
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

### CameraZoomLink — camera zoom bridge
- `scroll(steps)` — panel-side: accumulate one signed multiplicative zoom step per wheel notch, wheel up = zoom in (matching the wheel binding that shares the − / + actions)
- `request_zoom(zoom)` — panel-side: queue an absolute preset snap (`0.05, 0.15, 0.3, 0.6, 1.0, 2.0, 4.0`); a snap outranks queued steps in the next drain
- `drain_pending()` — host-side: `CameraZoomCommand::Steps(n)` (apply through `Camera::zoom_in`/`zoom_out`, which clamp) / `Set(zoom)` (clamp host-side to `MIN_ZOOM..MAX_ZOOM`) / `None`
- `set_current(zoom)` / `current()` — host publishes the live `Camera::zoom`; the zoom row displays it

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
  - validates row hit-testing, wheel fine-tune/clamp, preset cycling, parallax toggle/strength rows, depth row scroll/reset/display through the link, layers row scroll/preset-cycle/display through its link, zoom row step/preset-snap/display through its link (steps drain exactly like the − / + key path; preset snaps outrank queued steps), fall-through off rows, gizmo behavior, ui-state persistence, and draw anchoring

## data
- none

## notes
- Rows stack downward from the top content row; draw output is module-local, events are screen-space (shared module convention).
- Depth edits intentionally route through the host (link, not direct camera mutation) so keys and canvas wheel-scroll stay the only other writers and the camera keeps a single owner.
- Depth follows the swing via `Camera::focus_depth`, exactly like `pan_focus_depth`, and persists with the camera's focus target in host session state.
- The layers row edits `Camera::visible_plane_radius` — the rendered-layer count on each side of the focus plane — which was previously host-fixed with no UI; it already persists with the camera in the renderer's ui-session-state, so user changes survive restarts with no new persistence fields.
- The zoom row edits `Camera::zoom` — the same value the − and + keys (and the wheel binding) drive — so the host drains through `Camera::zoom_in`/`zoom_out` and a single owner stays intact; zoom persists with the camera in host session state, so no new persistence fields.
- The `depth` row stays the focus-plane center; the `layers` row is the window size. Two separate knobs on purpose.
