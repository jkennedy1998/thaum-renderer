//! Shared draggable scrollbar primitive for scrollable panels.
//!
//! Owns everything a scrollbar is: where it lives (`ScrollbarSpan`, either
//! horizontal or vertical, at a consumer-declared position), the thumb math
//! over content/viewport/offset, track + thumb cell rendering with J's
//! graphical preset slots (bar filled / bar empty / bar end caps) that
//! consumers may override, thumb-drag and track-click pointer behavior,
//! hover/drag active feedback, and the axis-aware wheel rule (a
//! horizontal-only bar takes vertical wheel input too, since most mice only
//! have a vertical wheel).
//!
//! The scrollbar is stateless about scroll position: the consumer keeps its
//! offset (usually in a `ScrollState`) and passes content/viewport/offset
//! into every call. Units are "content units" — rows for list panels,
//! buttons for the command bar — the math only uses their ratios against
//! the track length in cells. All coordinates are module-local draw coords,
//! the same space a module's own `Cell` positions use.

use crate::{Cell, CellColor, CellGraphic, CellPoint, CellWeight, UiColorRole, UiPalette};

/// Where a scrollbar lives, in module-local draw coordinates. Endpoints are
/// inclusive; the track covers every cell from start to finish. Per J: a
/// consumer declares a left point and a right point for horizontal, a top
/// point and a bottom point for vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarSpan {
    /// A horizontal bar on row `y`, spanning columns `x0..=x1` (left → right).
    Horizontal { y: i32, x0: i32, x1: i32 },
    /// A vertical bar on column `x`. `top` is the bar's screen-top end and
    /// `bottom` its screen-bottom end; Flat2d renders larger local y higher,
    /// so `top >= bottom` and the track walks downward from `top`. Offset 0
    /// (content start) puts the thumb at the top end.
    Vertical { x: i32, top: i32, bottom: i32 },
}

impl ScrollbarSpan {
    pub fn axis(&self) -> ScrollbarAxis {
        match self {
            ScrollbarSpan::Horizontal { .. } => ScrollbarAxis::Horizontal,
            ScrollbarSpan::Vertical { .. } => ScrollbarAxis::Vertical,
        }
    }

    /// Track length in cells (inclusive endpoints).
    pub fn track_len(&self) -> i32 {
        match self {
            ScrollbarSpan::Horizontal { x0, x1, .. } => x1 - x0 + 1,
            ScrollbarSpan::Vertical { top, bottom, .. } => top - bottom + 1,
        }
    }

    /// Whether a module-local pointer position is on the track.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        match self {
            ScrollbarSpan::Horizontal { y: bar_y, x0, x1 } => {
                y == *bar_y && (*x0..=*x1).contains(&x)
            }
            ScrollbarSpan::Vertical { x: bar_x, top, bottom } => {
                x == *bar_x && (*bottom..=*top).contains(&y)
            }
        }
    }

    /// Pointer position projected onto the track axis, 0 at the start end
    /// (left for horizontal, screen-top for vertical).
    pub fn position_along(&self, x: i32, y: i32) -> i32 {
        match self {
            ScrollbarSpan::Horizontal { x0, .. } => x - x0,
            ScrollbarSpan::Vertical { top, .. } => top - y,
        }
    }

    fn cell_position(&self, along: i32) -> CellPoint {
        match self {
            ScrollbarSpan::Horizontal { y, x0, .. } => CellPoint {
                x: x0 + along,
                y: *y,
                z: 0,
            },
            ScrollbarSpan::Vertical { x, top, .. } => CellPoint {
                x: *x,
                y: top - along,
                z: 0,
            },
        }
    }
}

/// Which way a scrollbar scrolls its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Horizontal,
    Vertical,
}

/// A draggable scrollbar: span (where it lives), thumb math, cell rendering,
/// and thumb-drag/track-click pointer behavior. Offset state stays with the
/// consumer; this type only reads it and writes back the new offset.
#[derive(Debug, Clone)]
pub struct Scrollbar {
    span: ScrollbarSpan,
    drag: Option<ThumbDrag>,
    /// Pointer currently over the bar's track (or a live drag session):
    /// drives the hover weight boost and the filled bar's vivid color.
    hovered: bool,
}

#[derive(Debug, Clone, Copy)]
struct ThumbDrag {
    /// Pointer offset inside the thumb at drag start, in track cells, held
    /// constant for the whole drag so the thumb moves with the pointer.
    grab: i32,
}

impl Scrollbar {
    pub fn new(span: ScrollbarSpan) -> Self {
        Self {
            span,
            drag: None,
            hovered: false,
        }
    }

    pub fn span(&self) -> &ScrollbarSpan {
        &self.span
    }

    /// Whether a module-local pointer position is on the bar's track.
    pub fn contains(&self, local_x: i32, local_y: i32) -> bool {
        self.span.contains(local_x, local_y)
    }

    /// Moves the bar to a new position without disturbing drag state, so a
    /// consumer can re-place the bar every frame as its rect resizes.
    pub fn set_span(&mut self, span: ScrollbarSpan) {
        self.span = span;
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Feeds a module-local pointer move: tracks whether the pointer sits on
    /// the bar, driving the hover look. Call from the consumer's Move arm.
    pub fn note_pointer(&mut self, local_x: i32, local_y: i32) {
        self.hovered = self.span.contains(local_x, local_y);
    }

    /// Pointer left the module: clears the hover look.
    pub fn clear_pointer(&mut self) {
        self.hovered = false;
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Hover or live drag: the bar's active state (weight boost, vivid fill).
    pub fn is_active(&self) -> bool {
        self.hovered || self.drag.is_some()
    }

    /// Whether the bar shows at all: only overflowing content scrolls.
    pub fn is_visible(content: usize, viewport: usize) -> bool {
        content > viewport
    }

    /// Largest scrollable offset for this content: how far past the viewport
    /// the content extends, saturated at 0.
    pub fn max_offset(content: usize, viewport: usize) -> usize {
        content.saturating_sub(viewport)
    }

    /// Thumb length in track cells, proportional to the visible fraction of
    /// the content, at least one cell, never longer than the track.
    pub fn thumb_len(track_len: i32, content: usize, viewport: usize) -> i32 {
        if content == 0 {
            return track_len;
        }
        let proportional =
            (track_len as f32 * viewport.min(content) as f32 / content as f32).round() as i32;
        proportional.clamp(1, track_len)
    }

    /// Thumb start position in track cells for the given offset, clamped so
    /// the thumb stays fully inside the track at both scroll edges.
    pub fn thumb_start(track_len: i32, content: usize, viewport: usize, offset: usize) -> i32 {
        let thumb_len = Self::thumb_len(track_len, content, viewport);
        let travel = track_len - thumb_len;
        let max_offset = Self::max_offset(content, viewport);
        if max_offset == 0 || travel <= 0 {
            return 0;
        }
        let start =
            (offset.min(max_offset) as f32 * travel as f32 / max_offset as f32).round() as i32;
        start.clamp(0, travel)
    }

    /// The offset that puts the thumb's leading edge at `thumb_pos` cells
    /// along the track (inverse of `thumb_start`), clamped to range.
    pub fn offset_at_thumb_start(
        thumb_pos: i32,
        track_len: i32,
        content: usize,
        viewport: usize,
    ) -> usize {
        let thumb_len = Self::thumb_len(track_len, content, viewport);
        let travel = track_len - thumb_len;
        let max_offset = Self::max_offset(content, viewport);
        if travel <= 0 || max_offset == 0 {
            return 0;
        }
        let offset =
            (thumb_pos.clamp(0, travel) as f32 * max_offset as f32 / travel as f32).round() as usize;
        offset.min(max_offset)
    }

    /// Track + thumb cells in module-local draw coordinates against the
    /// bar's current span. Empty when the content fits (nothing to scroll).
    /// Default graphics come from the palette; `graphics` overrides every
    /// glyph/color (the color block's custom look), while the hover weight
    /// boost always applies. `active` (pointer on the bar, or a live drag)
    /// boosts every preset's weight by one and shows the filled bar vivid.
    pub fn cells(
        &self,
        content: usize,
        viewport: usize,
        offset: usize,
        palette: &UiPalette,
        graphics: Option<&ScrollbarGraphics>,
        active: bool,
    ) -> Vec<Cell> {
        self.cells_with_span(self.span, content, viewport, offset, palette, graphics, active)
    }

    /// The same rendering against a caller-supplied span, for `&self` draw
    /// paths that derive the bar's placement fresh each frame instead of
    /// waiting for the next pointer event to re-place it — a module that
    /// boots or restores its rect into a scrollable shape renders a full
    /// bar immediately, never a stale leftover span.
    pub fn cells_with_span(
        &self,
        span: ScrollbarSpan,
        content: usize,
        viewport: usize,
        offset: usize,
        palette: &UiPalette,
        graphics: Option<&ScrollbarGraphics>,
        active: bool,
    ) -> Vec<Cell> {
        if !Self::is_visible(content, viewport) {
            return Vec::new();
        }
        let track_len = span.track_len();
        let look = graphics.cloned().unwrap_or_else(|| {
            ScrollbarGraphics::from_palette(palette, span.axis(), active)
        });
        let thumb_len = Self::thumb_len(track_len, content, viewport);
        let thumb_start = Self::thumb_start(track_len, content, viewport, offset);
        (0..track_len)
            .map(|along| {
                let in_thumb = (thumb_start..thumb_start + thumb_len).contains(&along);
                // Graphical preset slots (J): bar filled / bar empty, plus
                // the bar's two end caps. A thumb cell over the interior
                // draws filled; a thumb cell over an end cap keeps the
                // cap's own glyph — J: replacing the cap glyph with the
                // filled bar read as a cropped scrollbar — and the cap's
                // weight boosts by one instead, so the occupied cap still
                // reads as part of the thumb without losing its shape.
                let on_cap = along == 0 || along == track_len - 1;
                let occupied_cap = in_thumb && on_cap;
                let slot = if along == 0 {
                    &look.start_end
                } else if along == track_len - 1 {
                    &look.end_end
                } else if in_thumb {
                    &look.filled
                } else {
                    &look.empty
                };
                Cell {
                    position: span.cell_position(along),
                    graphic: CellGraphic::Glyph(slot.glyph),
                    color: slot.color.clone(),
                    // Occupied end cap and hover/drag each boost every
                    // preset's weight by one (J), clamped by CellWeight.
                    weight: CellWeight::from_index_clamped(
                        slot.weight.as_index() as i32
                            + occupied_cap as i32
                            + active as i32,
                    ),
                    ..Cell::default()
                }
            })
            .collect()
    }

    /// Pointer press on the bar. On the thumb this starts a drag anchored at
    /// the grab point; on the track it page-jumps so the thumb centers under
    /// the pointer and then starts a drag from there. Returns whether the
    /// press landed on a visible, scrollable bar (the consumer should treat
    /// that as consuming the click and capture the pointer).
    pub fn on_down(
        &mut self,
        x: i32,
        y: i32,
        content: usize,
        viewport: usize,
        offset: &mut usize,
    ) -> bool {
        if !Self::is_visible(content, viewport) || !self.span.contains(x, y) {
            return false;
        }
        let track_len = self.span.track_len();
        let thumb_len = Self::thumb_len(track_len, content, viewport);
        let rel = self.span.position_along(x, y);
        let thumb_start = Self::thumb_start(track_len, content, viewport, *offset);
        let grab = if (thumb_start..thumb_start + thumb_len).contains(&rel) {
            // Thumb grab: keep the pointer's position inside the thumb.
            rel - thumb_start
        } else {
            // Track jump: center the thumb under the pointer, then drag.
            *offset =
                Self::offset_at_thumb_start(rel - thumb_len / 2, track_len, content, viewport);
            thumb_len / 2
        };
        self.drag = Some(ThumbDrag { grab });
        true
    }

    /// Pointer move while dragging: re-map the grab point to a new offset.
    /// Returns whether the offset changed.
    pub fn on_drag(
        &mut self,
        x: i32,
        y: i32,
        content: usize,
        viewport: usize,
        offset: &mut usize,
    ) -> bool {
        let Some(drag) = self.drag else {
            return false;
        };
        let track_len = self.span.track_len();
        let thumb_len = Self::thumb_len(track_len, content, viewport);
        let travel = track_len - thumb_len;
        let max_offset = Self::max_offset(content, viewport);
        let rel = self.span.position_along(x, y);
        let raw = (rel - drag.grab).max(0) as f32;
        let next = if travel > 0 && max_offset > 0 {
            ((raw / travel as f32) * max_offset as f32).round() as usize
        } else {
            0
        }
        .min(max_offset);
        let moved = next != *offset;
        *offset = next;
        moved
    }

    /// Pointer release: ends the drag session.
    pub fn on_up(&mut self) {
        self.drag = None;
    }

    /// Axis-aware wheel delta in scroll steps, positive toward the track's
    /// end. Per J: a horizontal-only bar takes either horizontal wheel
    /// (mice that have one) or vertical wheel; a vertical bar takes vertical
    /// wheel. Consumes only the matching axis when both move at once.
    pub fn wheel_steps(delta_x: f32, delta_y: f32, axis: ScrollbarAxis) -> f32 {
        match axis {
            ScrollbarAxis::Horizontal => {
                if delta_x != 0.0 {
                    delta_x
                } else {
                    -delta_y
                }
            }
            ScrollbarAxis::Vertical => -delta_y,
        }
    }
}

/// One scrollbar preset slot: the glyph, color, and weight a stretch of bar
/// draws with. J owns the naming: bar filled, bar empty, and the bar's end
/// caps, so consumers can restyle the whole bar by preset (the color block
/// overrides these) while the shared seam keeps one default look.
#[derive(Debug, Clone, PartialEq)]
pub struct BarLook {
    pub glyph: char,
    pub color: CellColor,
    pub weight: CellWeight,
}

impl BarLook {
    fn solid(glyph: char, palette: &UiPalette, role: UiColorRole, weight_index: i32) -> Self {
        Self {
            glyph,
            color: palette.get(role),
            weight: CellWeight::from_index_clamped(weight_index),
        }
    }
}

/// The full look of one scrollbar, expressed in the graphical preset slots.
/// Horizontal bars use the horizontal glyph pair; vertical bars the
/// vertical one; `start_end`/`end_end` bookend the bar (left/right or
/// top/bottom). Defaults come from the UI palette.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollbarGraphics {
    /// Bar filled: the thumb stretch.
    pub filled: BarLook,
    /// Bar empty: the track between the caps.
    pub empty: BarLook,
    /// Bar start end: left end (horizontal) or top end (vertical).
    pub start_end: BarLook,
    /// Bar end end: right end (horizontal) or bottom end (vertical).
    pub end_end: BarLook,
}

impl ScrollbarGraphics {
    /// Default look, J's graphics: filled bar ━/┃ weight two (bright, vivid
    /// on hover), empty bar ═/║ and the end caps ┣┫/┳┻ weight one (medium).
    pub fn from_palette(palette: &UiPalette, axis: ScrollbarAxis, active: bool) -> Self {
        let (empty_glyph, filled_glyph, start_glyph, end_glyph) = match axis {
            ScrollbarAxis::Horizontal => ('═', '━', '┣', '┫'),
            ScrollbarAxis::Vertical => ('║', '┃', '┳', '┻'),
        };
        let filled_role = if active {
            UiColorRole::Vivid
        } else {
            UiColorRole::Bright
        };
        Self {
            filled: BarLook::solid(filled_glyph, palette, filled_role, 2),
            empty: BarLook::solid(empty_glyph, palette, UiColorRole::Medium, 1),
            start_end: BarLook::solid(start_glyph, palette, UiColorRole::Medium, 1),
            end_end: BarLook::solid(end_glyph, palette, UiColorRole::Medium, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> UiPalette {
        UiPalette::default()
    }

    fn h_span() -> ScrollbarSpan {
        ScrollbarSpan::Horizontal { y: 5, x0: 2, x1: 9 }
    }

    fn v_span() -> ScrollbarSpan {
        ScrollbarSpan::Vertical { x: 7, top: 10, bottom: 3 }
    }

    #[test]
    fn only_overflowing_content_shows_a_bar() {
        assert!(!Scrollbar::is_visible(10, 10));
        assert!(!Scrollbar::is_visible(5, 10));
        assert!(Scrollbar::is_visible(11, 10));
        assert_eq!(Scrollbar::max_offset(5, 10), 0);
        assert_eq!(Scrollbar::max_offset(13, 10), 3);
    }

    #[test]
    fn thumb_len_is_proportional_and_at_least_one_cell() {
        assert_eq!(Scrollbar::thumb_len(8, 100, 25), 2);
        assert_eq!(Scrollbar::thumb_len(8, 100, 4), 1);
        assert_eq!(Scrollbar::thumb_len(8, 10, 10), 8);
        assert_eq!(Scrollbar::thumb_len(8, 0, 0), 8);
    }

    #[test]
    fn thumb_start_clamps_at_both_scroll_edges() {
        // 4 content, 1 viewport over an 8-cell track: thumb 2, travel 6.
        assert_eq!(Scrollbar::thumb_start(8, 4, 1, 0), 0);
        assert_eq!(Scrollbar::thumb_start(8, 4, 1, 2), 4);
        assert_eq!(Scrollbar::thumb_start(8, 4, 1, 3), 6);
        assert_eq!(Scrollbar::thumb_start(8, 4, 1, 99), 6);
    }

    #[test]
    fn offset_at_thumb_start_round_trips_thumb_start() {
        for offset in 0..=3 {
            let pos = Scrollbar::thumb_start(8, 4, 1, offset);
            assert_eq!(Scrollbar::offset_at_thumb_start(pos, 8, 4, 1), offset);
        }
    }

    #[test]
    fn horizontal_bar_draws_track_and_thumb_cells() {
        let bar = Scrollbar::new(h_span());
        let cells = bar.cells(4, 1, 0, &palette(), None, false);
        assert_eq!(cells.len(), 8);
        let thumb = Scrollbar::thumb_len(8, 4, 1) as usize;
        let first = &cells[0];
        assert_eq!(first.position, CellPoint { x: 2, y: 5, z: 0 });
        // Offset 0 puts the thumb over the start cap: the cap keeps its own
        // glyph (J: swapping in the filled glyph read as a cropped bar) and
        // its weight boosts by one instead, 1 -> 2, color stays medium.
        assert_eq!(first.graphic, CellGraphic::Glyph('┣'));
        assert_eq!(first.weight, CellWeight::from_index_clamped(2));
        assert_eq!(first.color, palette().get(UiColorRole::Medium));
        // Interior thumb cell and empty track: the default looks.
        assert_eq!(cells[1].graphic, CellGraphic::Glyph('━'));
        assert_eq!(cells[1].weight, CellWeight::from_index_clamped(2));
        assert_eq!(cells[1].color, palette().get(UiColorRole::Bright));
        assert_eq!(cells[thumb].graphic, CellGraphic::Glyph('═'));
        assert_eq!(cells[thumb].weight, CellWeight::from_index_clamped(1));
        assert_eq!(cells[thumb].color, palette().get(UiColorRole::Medium));
        assert_eq!(cells[7].graphic, CellGraphic::Glyph('┫'));
    }

    #[test]
    fn vertical_bar_draws_down_from_its_top_end() {
        let bar = Scrollbar::new(v_span());
        let cells = bar.cells(4, 1, 0, &palette(), None, false);
        assert_eq!(cells.len(), 8);
        // Offset 0 puts the thumb at the screen-top end (larger y first):
        // the occupied top cap keeps its glyph, weight boosted 1 -> 2.
        assert_eq!(cells[0].position, CellPoint { x: 7, y: 10, z: 0 });
        assert_eq!(cells[0].graphic, CellGraphic::Glyph('┳'));
        assert_eq!(cells[0].weight, CellWeight::from_index_clamped(2));
        assert_eq!(cells[7].graphic, CellGraphic::Glyph('┻'));
        assert_eq!(cells[7].weight, CellWeight::from_index_clamped(1));
        // Offset 3 of 3: the thumb walks to the bottom end (rows 6..=7) —
        // the occupied bottom cap keeps its glyph, weight boosted 1 -> 2,
        // and the interior thumb cell above it draws the filled bar.
        let cells = bar.cells(4, 1, 3, &palette(), None, false);
        assert_eq!(cells[0].graphic, CellGraphic::Glyph('┳'));
        assert_eq!(cells[6].graphic, CellGraphic::Glyph('┃'));
        assert_eq!(cells[7].graphic, CellGraphic::Glyph('┻'));
        assert_eq!(cells[7].weight, CellWeight::from_index_clamped(2));
    }

    #[test]
    fn fitted_content_draws_no_cells() {
        let bar = Scrollbar::new(h_span());
        assert!(bar.cells(10, 10, 0, &palette(), None, false).is_empty());
    }

    #[test]
    fn hover_boosts_every_preset_weight_and_lights_the_fill() {
        let bar = Scrollbar::new(v_span());
        // Offset 1 puts the thumb in the track interior (cells 2..=3), so
        // neither cap is occupied and the default slots show.
        let idle = bar.cells(4, 1, 1, &palette(), None, false);
        let hover = bar.cells(4, 1, 1, &palette(), None, true);
        // Filled bar: weight 2 -> 3, bright -> vivid.
        assert_eq!(idle[2].weight, CellWeight::from_index_clamped(2));
        assert_eq!(hover[2].weight, CellWeight::from_index_clamped(3));
        assert_eq!(hover[2].color, palette().get(UiColorRole::Vivid));
        // Empty track: weight 1 -> 2, medium stays.
        assert_eq!(idle[7].weight, CellWeight::from_index_clamped(1));
        assert_eq!(hover[7].weight, CellWeight::from_index_clamped(2));
        assert_eq!(hover[7].color, palette().get(UiColorRole::Medium));
    }

    #[test]
    fn pointer_tracking_drives_the_active_look() {
        let mut bar = Scrollbar::new(v_span());
        assert!(!bar.is_active());
        bar.note_pointer(7, 10);
        assert!(bar.is_hovered());
        assert!(bar.is_active(), "hover counts as active");
        bar.note_pointer(0, 0);
        assert!(!bar.is_hovered(), "pointer off the track clears hover");
        bar.clear_pointer();
        assert!(!bar.is_active());
        // A live drag is active even with the pointer elsewhere.
        let mut offset = 0usize;
        bar.on_down(7, 10, 4, 1, &mut offset);
        bar.note_pointer(0, 0);
        assert!(bar.is_active(), "drag counts as active");
    }

    #[test]
    fn consumer_graphics_override_the_default_look() {
        let bar = Scrollbar::new(h_span());
        let graphics = ScrollbarGraphics {
            filled: BarLook {
                glyph: '◆',
                color: CellColor::Flat([1.0, 0.0, 1.0, 1.0]),
                weight: CellWeight::from_index_clamped(2),
            },
            empty: BarLook {
                glyph: ' ',
                color: CellColor::Flat([0.0, 0.0, 0.0, 1.0]),
                weight: CellWeight::from_index_clamped(0),
            },
            start_end: BarLook {
                glyph: '◄',
                color: CellColor::Flat([1.0, 1.0, 0.0, 1.0]),
                weight: CellWeight::from_index_clamped(1),
            },
            end_end: BarLook {
                glyph: '►',
                color: CellColor::Flat([0.0, 1.0, 1.0, 1.0]),
                weight: CellWeight::from_index_clamped(1),
            },
        };
        let cells = bar.cells(4, 1, 0, &palette(), Some(&graphics), false);
        // Offset 0: the thumb leads, so the occupied start cap keeps its
        // override glyph ('◄') and boosts weight 1 -> 2 with the override
        // color; the end cap bookends the empty track.
        assert_eq!(cells[0].graphic, CellGraphic::Glyph('◄'));
        assert_eq!(cells[0].color, CellColor::Flat([1.0, 1.0, 0.0, 1.0]));
        assert_eq!(cells[0].weight, CellWeight::from_index_clamped(2));
        let thumb = Scrollbar::thumb_len(8, 4, 1) as usize;
        assert_eq!(cells[thumb].graphic, CellGraphic::Glyph(' '));
        assert_eq!(cells[thumb].color, CellColor::Flat([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(cells[7].graphic, CellGraphic::Glyph('►'));
        assert_eq!(cells[7].color, CellColor::Flat([0.0, 1.0, 1.0, 1.0]));
        // The override keeps its colors under hover, but the weight boost
        // still applies (J: same weight animation for every bar).
        let hover = bar.cells(4, 1, 0, &palette(), Some(&graphics), true);
        assert_eq!(hover[0].color, CellColor::Flat([1.0, 1.0, 0.0, 1.0]));
        assert_eq!(hover[0].weight, CellWeight::from_index_clamped(3));
        assert_eq!(hover[7].weight, CellWeight::from_index_clamped(2));
    }

    #[test]
    fn an_occupied_end_cap_keeps_its_glyph_and_boosts_weight() {
        let bar = Scrollbar::new(v_span());
        // Offset 3 of 3 parks the thumb on the bottom end: the occupied
        // bottom cap draws ┻ (never the filled ┃), weight 1 + 1 = 2, and
        // the interior thumb cell above it stays the filled bar.
        let cells = bar.cells(4, 1, 3, &palette(), None, false);
        assert_eq!(cells[7].graphic, CellGraphic::Glyph('┻'));
        assert_eq!(cells[7].weight, CellWeight::from_index_clamped(2));
        assert_eq!(cells[7].color, palette().get(UiColorRole::Medium));
        assert_eq!(cells[6].graphic, CellGraphic::Glyph('┃'));
        // The occupied-cap boost stacks with the hover boost, clamped.
        let hover = bar.cells(4, 1, 3, &palette(), None, true);
        assert_eq!(hover[7].weight, CellWeight::from_index_clamped(3));
    }

    #[test]
    fn thumb_drag_maps_pointer_positions_to_offsets() {
        let mut bar = Scrollbar::new(h_span());
        let mut offset = 0usize;
        // Press on the thumb's leading edge (thumb len 2 at offset 0).
        assert!(bar.on_down(2, 5, 4, 1, &mut offset));
        assert!(bar.is_dragging());
        // Drag the grab point to the far end of the track.
        bar.on_drag(9, 5, 4, 1, &mut offset);
        assert_eq!(offset, 3);
        // Drag back to the start.
        bar.on_drag(2, 5, 4, 1, &mut offset);
        assert_eq!(offset, 0);
        bar.on_up();
        assert!(!bar.is_dragging());
        // Drags without a drag session are inert.
        assert!(!bar.on_drag(5, 5, 4, 1, &mut offset));
        assert_eq!(offset, 0);
    }

    #[test]
    fn track_click_jumps_the_thumb_under_the_pointer() {
        let mut bar = Scrollbar::new(h_span());
        let mut offset = 0usize;
        // Press the far end of the track: thumb centers there.
        assert!(bar.on_down(9, 5, 4, 1, &mut offset));
        assert_eq!(offset, 3);
        // The press became a drag anchored mid-thumb, so the thumb follows.
        bar.on_drag(6, 5, 4, 1, &mut offset);
        assert_eq!(offset, 2);
    }

    #[test]
    fn pointer_off_the_bar_is_not_consumed() {
        let mut bar = Scrollbar::new(h_span());
        let mut offset = 0usize;
        assert!(!bar.on_down(5, 4, 4, 1, &mut offset));
        assert!(!bar.on_down(20, 5, 4, 1, &mut offset));
        assert!(!bar.on_down(5, 5, 10, 10, &mut offset));
        assert!(!bar.is_dragging());
    }

    #[test]
    fn set_span_replaces_geometry_but_keeps_a_live_drag() {
        let mut bar = Scrollbar::new(h_span());
        let mut offset = 0usize;
        bar.on_down(2, 5, 4, 1, &mut offset);
        bar.set_span(v_span());
        assert!(bar.is_dragging());
        // The drag continues against the new geometry: top-end-relative rel
        // 2 with grab 0 maps to round(2/6 * 3) = 1.
        bar.on_drag(7, 8, 4, 1, &mut offset);
        assert_eq!(offset, 1);
    }

    #[test]
    fn cells_with_span_matches_a_reset_span() {
        let mut bar = Scrollbar::new(h_span());
        let fresh = bar.cells_with_span(v_span(), 4, 1, 0, &palette(), None, false);
        bar.set_span(v_span());
        assert_eq!(bar.cells(4, 1, 0, &palette(), None, false), fresh);
    }

    #[test]
    fn horizontal_wheel_takes_either_wheel_axis() {
        // Horizontal wheel wins when it moves.
        assert_eq!(
            Scrollbar::wheel_steps(1.0, 2.0, ScrollbarAxis::Horizontal),
            1.0
        );
        assert_eq!(
            Scrollbar::wheel_steps(-2.0, 0.0, ScrollbarAxis::Horizontal),
            -2.0
        );
        // Vertical wheel scrolls a horizontal bar (down = toward the end).
        assert_eq!(
            Scrollbar::wheel_steps(0.0, -1.0, ScrollbarAxis::Horizontal),
            1.0
        );
        assert_eq!(
            Scrollbar::wheel_steps(0.0, 1.0, ScrollbarAxis::Horizontal),
            -1.0
        );
        // Vertical bars take only vertical wheel (up = toward the start).
        assert_eq!(Scrollbar::wheel_steps(1.0, 0.0, ScrollbarAxis::Vertical), 0.0);
        assert_eq!(
            Scrollbar::wheel_steps(0.0, 1.0, ScrollbarAxis::Vertical),
            -1.0
        );
    }
}
