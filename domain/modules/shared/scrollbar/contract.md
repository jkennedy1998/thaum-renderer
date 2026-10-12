# thaum-renderer/domain/modules/shared/scrollbar

## purpose
Own the shared draggable scrollbar primitive so every scrolling panel and the command bar show the same, tight scroll feel: thumb math over content/viewport/offset, track + thumb rendering with a default palette graphic consumers may override, thumb-drag and track-click pointer behavior, and the axis-aware wheel rule.

## owns
- the `Scrollbar` type and `ScrollbarSpan` placement (horizontal or vertical, at a consumer-declared track, endpoints inclusive, module-local draw coordinates; horizontal declares left/right points, vertical declares top/bottom points with Flat2d top = larger y)
- thumb math: proportional thumb length, clamped thumb position, offset ↔ thumb-position mapping
- track + thumb cell rendering as graphical preset slots (J): bar filled, bar empty, bar start end, bar end end — default look from `ui-palette/`, fully overridable per consumer via `ScrollbarGraphics`/`BarLook`; a thumb cell over an end cap keeps the cap's own glyph and boosts the cap's weight by one (J: swapping in the filled glyph made the bar look cropped)
- the hover/drag active feedback (J): every preset's weight boosts by one while hovered or dragged and returns when idle; the default filled bar also goes bright → vivid on hover/drag; consumer color overrides keep their colors but share the weight animation
- pointer behavior: thumb grab-and-drag, track click = page jump then drag, release ends the drag
- `set_span` re-placement that keeps a live drag, so consumers can re-place the bar every frame while resizing
- the wheel rule (J): a horizontal-only bar takes horizontal wheel when the mouse has one and falls back to vertical wheel; vertical bars take vertical wheel
- visibility rule: content must overflow the viewport for the bar to draw

## does not own
- scroll offset state (usually a `ScrollState`), owned by the consumer; the scrollbar only reads offset and writes back new offsets
- what the content is, crop logic, or wheel routing, owned by each consuming module
- pointer capture dispatch, owned by `domain/modules/` and each consumer's drag-session plumbing
- the command bar's button layout, owned by `domain/command-bar/`

## children-encapsulations
- none

## contents
- `scrollbar.rs`
  - `Scrollbar`, `ScrollbarSpan`, `ScrollbarAxis`, `ScrollbarGraphics`, and the thumb/track/drag/wheel math

## dependencies
- `thaum-renderer/domain/modules/shared/ui-palette/`

## exposed interfaces
- `Scrollbar::new(span)` — build one bar at a declared span
- `Scrollbar::set_span(span)` — re-place the bar, keeping any live drag
- `Scrollbar::is_visible(content, viewport)` / `max_offset(content, viewport)` — overflow and travel math
- `Scrollbar::thumb_len/thumb_start/offset_at_thumb_start` — pure thumb-position math
- `Scrollbar::cells(..., active)` — track + thumb cells in module-local coords against the bar's current span, default or preset-overridden graphics with the active weight boost
- `Scrollbar::cells_with_span(span, ..., active)` — the same rendering against a caller-supplied span, for `&self` draw paths that derive the bar's placement fresh each frame (a boot/restore render is never a stale leftover span)
- `Scrollbar::on_down/on_drag/on_up` — thumb grab, track page-jump, drag mapping, release
- `Scrollbar::wheel_steps(delta_x, delta_y, axis)` — axis-aware wheel delta, positive toward the track end
- `ScrollbarGraphics`/`BarLook::from_palette(palette, axis, active)` — the preset-slot look: filled/empty/start end/end end, axis-paired glyphs
- `Scrollbar::note_pointer/clear_pointer/is_hovered/is_active` — hover tracking that drives the active look

## interface consumers
- `thaum-renderer/domain/command-bar/`: horizontal bar over the button list (wheel + drag)
- `thaum-renderer/domain/modules/individuals/controls-panel/`: vertical bar over the slot list
- `thaum-renderer/domain/modules/individuals/camera-perspective/`: vertical bar over the knob/value band
- `thaum-painter/domain/modules/individuals/graphic-picker/`: vertical bar over the category rows
- `thaum-painter/domain/modules/individuals/hand-settings/`: vertical bar over the property rows

## artifacts
- none

## tests
- `scrollbar`
  - light
  - overflow visibility, proportional thumb length, clamped thumb start, offset round-trip, default and preset-overridden cell rendering (incl. end caps and occupied-cap weight), cells_with_span equivalence, thumb drag mapping, track page-jump, off-bar rejection, span re-placement mid-drag, axis-aware wheel rule

## data
- none

## notes
- J: "We have shared implementation for buttons I think it would make sense to have a shared implementation for scroll bars. I think a shared implementation for horizontal scroll bar and vertical scroll bar is a good way to split it."
- J: horizontal and vertical are one seam split into two span shapes — the math is axis-generic, so there is no duplicated scrollbar logic anywhere.
- J: bars are placed dynamically by the consumer — next to a side (top/bottom/left/right, just inside the border) or at a custom location declared as start/finish points along the bar's axis.
- J: "We do support scrolling left and right for mouses that have a horizontal scroll wheel" and a single horizontal bar should take vertical wheel input too — encoded in `wheel_steps`.
- J: "bar filled horizontal ━ weight 2 / bar filled vertical ┃ weight 2 / bar empty vertical ║ weight 1 / bar empty horizontal ═ weight 1 / bar left end ┣ weight 1 / bar right end ┫ weight 1 / bar top end ┳ weight 1 / bar bottom end ┻ weight 1" — the default preset graphics, exact glyphs and weights.
- J: "ideally these increase weight by 1 when hovered / dragged and return back to normal when non hover" and "when the user is hovering on the bar area the filled bar can go to vivid" — the active feedback baked into `cells`.
- J: "lets just increase the weight of the endcaps of the scrollbar graphics by 1 when they are filled, instead of replacing their graphic with ┃ or ━ bc that makes it look liek the scrollbar is cropped" — occupied end caps keep their glyph, weight +1; the occupied-cap boost stacks with the hover/drag boost.
- J: "lets have the filled bars be bright, and the non filled bars / ends be medium."
- J: "this coloring should be overriden in the color block module bc it uses different graphics and colors, but the weight can be the same animation" — overrides replace glyphs/colors; the weight boost stays seam-owned.
- J: "i think ideally all scroll bars use these shapes of pieces tho, that makes interaction easy."
- J names eight graphical presets — bar filled horizontal, bar filled vertical, bar empty vertical, bar empty horizontal, bar left end, bar right end, bar top end, bar bottom end — so every bar shares the preset slots and consumers (the color block) override graphics without touching the seam. The axis picks which filled/empty pair renders; start/end caps map to left/right or top/bottom.
- Units are content units, not cells: rows for list panels, buttons for the command bar; only ratios against the track length matter.
