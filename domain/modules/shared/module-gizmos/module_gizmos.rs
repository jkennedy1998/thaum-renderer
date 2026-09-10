use crate::{
    Cell, CellColor, CellGraphic, CellPoint, CellWeight, Hotspot, ModuleRect, PanelBorderEdge,
    PanelChrome, UiColorRole, UiPalette,
};

/// One gizmo button a module can offer in its top-left gizmo bar. Ported
/// from the old mono_ui `GizmoType`, minus `save_position` (no consumer
/// needs it yet).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GizmoKind {
    Move,
    Close,
    Resize,
    Seamless,
}

impl GizmoKind {
    fn glyph(self) -> char {
        match self {
            GizmoKind::Move => '#',
            GizmoKind::Close => 'X',
            GizmoKind::Resize => '╋',
            GizmoKind::Seamless => 'S',
        }
    }

    /// Tooltip title: the gizmo's one-word name.
    fn tooltip_title(self) -> &'static str {
        match self {
            GizmoKind::Move => "move",
            GizmoKind::Close => "close",
            GizmoKind::Resize => "resize",
            GizmoKind::Seamless => "seamless",
        }
    }

    /// Tooltip description: how to interact with the gizmo.
    fn tooltip_description(self) -> &'static str {
        match self {
            GizmoKind::Move => "drag to reposition this panel",
            GizmoKind::Close => "closes the module can be re-opened",
            GizmoKind::Resize => "click, then grab a border edge",
            GizmoKind::Seamless => "hides the panel chrome, hovering brings it back",
        }
    }
}

/// What a click into the gizmo interaction area resolved to — either a
/// gizmo glyph itself, or (once resize mode is armed) one of the panel's
/// four border edges, grabbed to start a per-edge resize drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoClickOutcome {
    Gizmo(GizmoKind),
    ResizeEdgeGrabbed(ResizeEdge),
}

/// Which border edge a resize drag is stretching. Ported from the old
/// mono_ui `ResizeEdge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl ResizeEdge {
    pub const fn border_edge(self) -> PanelBorderEdge {
        match self {
            ResizeEdge::Left => PanelBorderEdge::Left,
            ResizeEdge::Right => PanelBorderEdge::Right,
            ResizeEdge::Top => PanelBorderEdge::Top,
            ResizeEdge::Bottom => PanelBorderEdge::Bottom,
        }
    }
}

/// Which edge of `rect`, if any, absolute point `(x, y)` lands exactly on
/// (corners excluded — they belong to neither edge). Ported from the old
/// mono_ui `get_resize_edge`.
fn resize_edge_at(rect: ModuleRect, x: i32, y: i32) -> Option<ResizeEdge> {
    let on_left = x == rect.x0;
    let on_right = x == rect.x1;
    let on_top = y == rect.y1;
    let on_bottom = y == rect.y0;

    if on_left && !on_top && !on_bottom {
        Some(ResizeEdge::Left)
    } else if on_right && !on_top && !on_bottom {
        Some(ResizeEdge::Right)
    } else if on_top && !on_left && !on_right {
        Some(ResizeEdge::Top)
    } else if on_bottom && !on_left && !on_right {
        Some(ResizeEdge::Bottom)
    } else {
        None
    }
}

/// The fixed, ordered set of gizmos a module offers. Drawn and hit-tested
/// on the panel's own top border row, one cell in from the left corner,
/// spaced two cells apart (one blank gap after each glyph) — matching the
/// old mono_ui `get_gizmo_rect`'s spacing.
pub struct GizmoBar {
    kinds: Vec<GizmoKind>,
}

impl GizmoBar {
    /// The standard gizmo bar every gizmo-enabled module ships: move, close,
    /// resize, seamless, in that order.
    pub fn standard() -> Self {
        Self::new(vec![
            GizmoKind::Move,
            GizmoKind::Close,
            GizmoKind::Resize,
            GizmoKind::Seamless,
        ])
    }

    pub fn new(kinds: Vec<GizmoKind>) -> Self {
        Self { kinds }
    }

    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    fn glyph_local_x(index: usize) -> i32 {
        1 + (index as i32) * 2
    }

    /// Where a title on the same border row must start to clear every
    /// gizmo glyph plus a one-cell gap — pass straight into
    /// `PanelChrome::with_title_start_x`.
    pub fn title_start_x(&self) -> i32 {
        Self::glyph_local_x(self.kinds.len())
    }

    /// Gizmo glyph cells, positioned local to `rect` (its top border row).
    /// Active gizmos (the pressed toggle state in `state`) draw in the
    /// palette's `vivid` role; idle ones draw in `medium`.
    pub fn cells(&self, rect: ModuleRect, state: &GizmoState, palette: &UiPalette) -> Vec<Cell> {
        let height = rect.y1 - rect.y0;
        self.kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| Cell {
                position: CellPoint {
                    x: Self::glyph_local_x(index),
                    y: height - 1,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(kind.glyph()),
                color: state.gizmo_color(*kind, palette),
                weight: CellWeight::from_index_clamped(state.gizmo_weight_index(*kind)),
                ..Cell::default()
            })
            .collect()
    }

    /// Which gizmo, if any, absolute point `(x, y)` lands on.
    pub fn hit_test(&self, rect: ModuleRect, x: i32, y: i32) -> Option<GizmoKind> {
        let height = rect.y1 - rect.y0;
        let local_x = x - rect.x0;
        let local_y = y - rect.y0;
        if local_y != height - 1 {
            return None;
        }
        self.kinds
            .iter()
            .enumerate()
            .find_map(|(index, kind)| (local_x == Self::glyph_local_x(index)).then_some(*kind))
    }

    /// Tooltip hotspots for this bar: one single-cell anchor per gizmo
    /// glyph on the panel's top border row, in absolute screen space. Every
    /// gizmo-enabled module surfaces these through `Module::hotspots`, so
    /// all of them grow tooltips at once.
    pub fn hotspots(&self, rect: ModuleRect) -> Vec<Hotspot> {
        self.hotspots_with(rect, Vec::new())
    }

    /// Gizmo-bar hotspots plus a module's own custom-control hotspots: the
    /// one seam every module's `Module::hotspots` override routes through, so
    /// custom gizmos grow tooltips from the same shared implementation
    /// instead of each module re-assembling the list.
    pub fn hotspots_with(&self, rect: ModuleRect, custom: Vec<Hotspot>) -> Vec<Hotspot> {
        let height = rect.y1 - rect.y0;
        self.kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| {
                let x = rect.x0 + Self::glyph_local_x(index);
                let y = rect.y0 + height - 1;
                Hotspot::new(
                    ModuleRect {
                        x0: x,
                        y0: y,
                        x1: x,
                        y1: y,
                    },
                    kind.tooltip_title(),
                    kind.tooltip_description(),
                )
            })
            .chain(custom)
            .collect()
    }
}

/// One hotspot over a panel's title text on its top border row (the same
/// row the gizmo glyphs sit on, starting at the bar's `title_start_x`).
/// The always-visible module name is the natural "what is this panel?"
/// anchor: hover it to learn what the module does (J 2026-09-10).
pub fn title_hotspot(
    rect: ModuleRect,
    title_start_x: i32,
    title: &str,
    description: impl Into<String>,
) -> Hotspot {
    let y = rect.y0 + (rect.y1 - rect.y0 - 1);
    let x0 = rect.x0 + title_start_x;
    Hotspot::new(
        ModuleRect {
            x0,
            y0: y,
            x1: x0 + title.chars().count() as i32 - 1,
            y1: y,
        },
        format!("{title} panel"),
        description,
    )
}

const MIN_DRAG_SIZE: i32 = 6;

/// Per-module gizmo interaction state: which toggles are active, whether the
/// pointer currently hovers the module (for seamless mode), and (while a
/// move or resize drag is in progress) the pointer-capture drag session. A
/// gizmo-enabled module owns one of these as a field alongside its own
/// `ModuleRect`, driving it from `Module::on_pointer_event`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GizmoState {
    move_mode: bool,
    resize_mode: bool,
    seamless: bool,
    hovered: bool,
    hovered_gizmo: Option<GizmoKind>,
    hovered_resize_edge: Option<ResizeEdge>,
    dragging: bool,
    drag_origin: (i32, i32),
    drag_original_rect: ModuleRect,
    resize_edge: Option<ResizeEdge>,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            move_mode: false,
            resize_mode: false,
            seamless: false,
            hovered: false,
            hovered_gizmo: None,
            hovered_resize_edge: None,
            dragging: false,
            drag_origin: (0, 0),
            drag_original_rect: ModuleRect {
                x0: 0,
                y0: 0,
                x1: 0,
                y1: 0,
            },
            resize_edge: None,
        }
    }
}

impl GizmoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_seamless(&self) -> bool {
        self.seamless
    }

    pub fn set_seamless(&mut self, seamless: bool) {
        self.seamless = seamless;
    }

    /// Whether the pointer is currently over the module (see
    /// `set_hovered`). A seamless module uses this to decide whether its
    /// gizmo bar should currently be visible.
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Drive hover state from the module's `Enter`/`Leave` pointer events.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
        if !hovered {
            self.hovered_gizmo = None;
            self.hovered_resize_edge = None;
        }
    }

    /// Drive fine-grained hover state from pointer motion over the module.
    pub fn note_pointer(&mut self, bar: &GizmoBar, rect: ModuleRect, x: i32, y: i32) {
        self.hovered_gizmo = bar.hit_test(rect, x, y);
        self.hovered_resize_edge = if self.resize_mode {
            resize_edge_at(rect, x, y)
        } else {
            None
        };
    }

    /// Whether the gizmo bar itself should currently be drawn: always, when
    /// not seamless; only while hovered, when seamless — so a seamless
    /// module can still be discovered and toggled back without needing
    /// permanently visible chrome.
    pub fn should_draw_gizmo_bar(&self) -> bool {
        !self.seamless || self.hovered
    }

    /// Whether the owning module should report `wants_pointer_capture() ==
    /// true` right now — true for the duration of a move/resize drag.
    pub fn wants_pointer_capture(&self) -> bool {
        self.dragging
    }

    fn is_active(&self, kind: GizmoKind) -> bool {
        match kind {
            GizmoKind::Move => self.move_mode,
            GizmoKind::Resize => self.resize_mode,
            GizmoKind::Seamless => self.seamless,
            GizmoKind::Close => false,
        }
    }

    fn is_hovered_gizmo(&self, kind: GizmoKind) -> bool {
        self.hovered_gizmo == Some(kind)
    }

    pub fn gizmo_color(&self, kind: GizmoKind, palette: &UiPalette) -> CellColor {
        if self.is_active(kind) || self.is_hovered_gizmo(kind) {
            palette.get(UiColorRole::Vivid)
        } else {
            palette.get(UiColorRole::Medium)
        }
    }

    pub fn gizmo_weight_index(&self, kind: GizmoKind) -> i32 {
        if self.is_active(kind) || self.is_hovered_gizmo(kind) {
            3
        } else {
            2
        }
    }

    pub fn border_color(&self, edge: ResizeEdge, palette: &UiPalette) -> CellColor {
        if self.move_mode {
            return palette.get(UiColorRole::Vivid);
        }
        if self.resize_mode {
            if self.resize_edge == Some(edge) || self.hovered_resize_edge == Some(edge) {
                return palette.get(UiColorRole::Vivid);
            }
            return palette.get(UiColorRole::Medium);
        }
        palette.get(UiColorRole::Dimmest)
    }

    pub fn border_weight_index(&self, edge: ResizeEdge) -> i32 {
        if self.move_mode {
            return 2;
        }
        if self.resize_mode {
            if self.resize_edge == Some(edge) || self.hovered_resize_edge == Some(edge) {
                return 3;
            }
            return 2;
        }
        1
    }

    pub fn decorate_panel_chrome(&self, chrome: PanelChrome, palette: &UiPalette) -> PanelChrome {
        [
            ResizeEdge::Left,
            ResizeEdge::Right,
            ResizeEdge::Top,
            ResizeEdge::Bottom,
        ]
        .into_iter()
        .fold(chrome.with_border_weight(1), |chrome, edge| {
            chrome
                .with_edge_color(edge.border_edge(), self.border_color(edge, palette))
                .with_edge_weight(edge.border_edge(), self.border_weight_index(edge))
        })
    }

    /// Handle a click at absolute `(x, y)`. First checks `bar`'s gizmo
    /// glyphs; if none hit and resize mode is currently armed, checks
    /// whether the click landed exactly on one of `rect`'s four border
    /// edges and, if so, grabs that edge to start a per-edge resize drag.
    /// Returns `None` if the click was neither — the caller should fall
    /// through to its own content hit-testing.
    pub fn handle_click(
        &mut self,
        bar: &GizmoBar,
        rect: ModuleRect,
        x: i32,
        y: i32,
    ) -> Option<GizmoClickOutcome> {
        if let Some(kind) = bar.hit_test(rect, x, y) {
            self.hovered_gizmo = Some(kind);
            match kind {
                GizmoKind::Move => {
                    self.resize_mode = false;
                    self.resize_edge = None;
                    self.move_mode = true;
                    self.dragging = true;
                    self.drag_origin = (x, y);
                    self.drag_original_rect = rect;
                }
                GizmoKind::Resize => {
                    self.move_mode = false;
                    // Resize only arms here; the actual drag starts once
                    // the user grabs one of the four border edges below.
                    self.resize_mode = !self.resize_mode;
                    self.dragging = false;
                    self.resize_edge = None;
                }
                GizmoKind::Seamless => {
                    self.seamless = !self.seamless;
                }
                GizmoKind::Close => {}
            }
            return Some(GizmoClickOutcome::Gizmo(kind));
        }

        if self.resize_mode && !self.dragging {
            if let Some(edge) = resize_edge_at(rect, x, y) {
                self.dragging = true;
                self.drag_origin = (x, y);
                self.drag_original_rect = rect;
                self.resize_edge = Some(edge);
                return Some(GizmoClickOutcome::ResizeEdgeGrabbed(edge));
            }
        }

        None
    }

    /// The rect that should now apply given the pointer at absolute `(x,
    /// y)`, if a move or per-edge resize drag is in progress. `None` once
    /// neither is active.
    pub fn drag_rect(&self, x: i32, y: i32) -> Option<ModuleRect> {
        if !self.dragging {
            return None;
        }
        let dx = x - self.drag_origin.0;
        let dy = y - self.drag_origin.1;
        let original = self.drag_original_rect;
        if self.move_mode {
            Some(ModuleRect {
                x0: original.x0 + dx,
                y0: original.y0 + dy,
                x1: original.x1 + dx,
                y1: original.y1 + dy,
            })
        } else if let Some(edge) = self.resize_edge {
            let mut next = original;
            match edge {
                ResizeEdge::Left => next.x0 = (original.x0 + dx).min(original.x1 - MIN_DRAG_SIZE),
                ResizeEdge::Right => next.x1 = (original.x1 + dx).max(original.x0 + MIN_DRAG_SIZE),
                ResizeEdge::Top => next.y1 = (original.y1 + dy).max(original.y0 + MIN_DRAG_SIZE),
                ResizeEdge::Bottom => next.y0 = (original.y0 + dy).min(original.y1 - MIN_DRAG_SIZE),
            }
            Some(next)
        } else {
            None
        }
    }

    /// End the current drag session (on pointer-up). Move is a one-shot drag
    /// that disarms immediately on release; resize stays armed so another edge
    /// can be grabbed without re-clicking its gizmo.
    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.move_mode = false;
        self.resize_edge = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> ModuleRect {
        ModuleRect { x0, y0, x1, y1 }
    }

    #[test]
    fn hotspots_anchor_one_cell_per_gizmo_glyph_with_copy() {
        let bar = GizmoBar::standard();
        let panel = rect(40, 10, 60, 24);
        let hotspots = bar.hotspots(panel);

        assert_eq!(hotspots.len(), 4);
        let height = panel.y1 - panel.y0;
        for (index, hotspot) in hotspots.iter().enumerate() {
            let expected_x = panel.x0 + 1 + index as i32 * 2;
            let expected_y = panel.y0 + height - 1;
            assert_eq!(
                hotspot.rect,
                rect(expected_x, expected_y, expected_x, expected_y)
            );
            assert!(!hotspot.title.is_empty());
            assert!(!hotspot.description.is_empty());
        }
        assert_eq!(hotspots[0].title, "move");
        assert_eq!(hotspots[1].title, "close");
        assert_eq!(hotspots[2].title, "resize");
        assert_eq!(hotspots[3].title, "seamless");
    }

    fn full_bar() -> GizmoBar {
        GizmoBar::standard()
    }

    #[test]
    fn standard_bar_ships_move_close_resize_seamless_in_order() {
        let bar = GizmoBar::standard();
        let rect = rect(0, 0, 20, 5);
        assert_eq!(bar.hit_test(rect, 1, 4), Some(GizmoKind::Move));
        assert_eq!(bar.hit_test(rect, 3, 4), Some(GizmoKind::Close));
        assert_eq!(bar.hit_test(rect, 5, 4), Some(GizmoKind::Resize));
        assert_eq!(bar.hit_test(rect, 7, 4), Some(GizmoKind::Seamless));
        assert_eq!(bar.title_start_x(), 1 + 4 * 2);
    }

    #[test]
    fn title_start_x_clears_every_gizmo_glyph_and_its_gap() {
        assert_eq!(full_bar().title_start_x(), 1 + 4 * 2);
        assert_eq!(GizmoBar::new(vec![]).title_start_x(), 1);
    }

    #[test]
    fn hit_test_finds_each_gizmo_by_its_two_cell_spacing() {
        let bar = full_bar();
        let r = rect(10, 10, 30, 20);

        assert_eq!(bar.hit_test(r, 11, 19), Some(GizmoKind::Move));
        assert_eq!(bar.hit_test(r, 13, 19), Some(GizmoKind::Close));
        assert_eq!(bar.hit_test(r, 15, 19), Some(GizmoKind::Resize));
        assert_eq!(bar.hit_test(r, 17, 19), Some(GizmoKind::Seamless));
        assert_eq!(bar.hit_test(r, 12, 19), None);
        assert_eq!(bar.hit_test(r, 11, 20), None);
    }

    #[test]
    fn clicking_move_starts_a_one_shot_drag_and_requests_capture() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();

        let clicked = state.handle_click(&bar, r, 1, 9);

        assert_eq!(clicked, Some(GizmoClickOutcome::Gizmo(GizmoKind::Move)));
        assert!(state.wants_pointer_capture());
    }

    #[test]
    fn dragging_after_move_click_translates_the_rect_by_the_pointer_delta() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 1, 9);

        let dragged = state
            .drag_rect(6, 13)
            .expect("expected an active move drag");

        assert_eq!(dragged, rect(5, 4, 25, 14));
    }

    #[test]
    fn ending_a_move_drag_disarms_it_immediately() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 1, 9);
        state.end_drag();

        assert!(!state.wants_pointer_capture());
        assert_eq!(state.drag_rect(50, 50), None);
    }

    #[test]
    fn clicking_resize_only_arms_it_without_starting_a_drag() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();

        let clicked = state.handle_click(&bar, r, 5, 9);

        assert_eq!(clicked, Some(GizmoClickOutcome::Gizmo(GizmoKind::Resize)));
        assert!(!state.wants_pointer_capture());
        assert_eq!(state.drag_rect(50, 50), None);
    }

    #[test]
    fn once_armed_grabbing_the_right_edge_resizes_only_that_edge() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9); // arm resize

        let grabbed = state.handle_click(&bar, r, 20, 5); // grab right edge
        assert_eq!(
            grabbed,
            Some(GizmoClickOutcome::ResizeEdgeGrabbed(ResizeEdge::Right))
        );
        assert!(state.wants_pointer_capture());

        let dragged = state
            .drag_rect(26, 5)
            .expect("expected an active resize drag");
        assert_eq!(dragged, rect(0, 0, 26, 10));
    }

    #[test]
    fn grabbing_the_left_edge_moves_only_x0_and_respects_a_minimum_width() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9);
        state.handle_click(&bar, r, 0, 5); // grab left edge

        let dragged = state
            .drag_rect(4, 5)
            .expect("expected an active resize drag");
        assert_eq!(dragged, rect(4, 0, 20, 10));

        let shrunk = state
            .drag_rect(100, 5)
            .expect("expected an active resize drag");
        assert_eq!(shrunk, rect(20 - MIN_DRAG_SIZE, 0, 20, 10));
    }

    #[test]
    fn grabbing_the_top_edge_moves_only_y1_and_the_bottom_edge_moves_only_y0() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9);

        state.handle_click(&bar, r, 10, 10); // grab top edge
        let grown_up = state
            .drag_rect(10, 16)
            .expect("expected an active resize drag");
        assert_eq!(grown_up, rect(0, 0, 20, 16));
        state.end_drag();

        state.handle_click(&bar, r, 10, 0); // grab bottom edge
        let grown_down = state
            .drag_rect(10, -4)
            .expect("expected an active resize drag");
        assert_eq!(grown_down, rect(0, -4, 20, 10));
    }

    #[test]
    fn a_click_inside_the_panel_while_resize_is_armed_grabs_nothing() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9);

        assert_eq!(state.handle_click(&bar, r, 10, 5), None);
        assert!(!state.wants_pointer_capture());
    }

    #[test]
    fn ending_a_resize_drag_keeps_resize_armed_for_another_edge_grab() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9);
        state.handle_click(&bar, r, 20, 5);
        state.end_drag();

        assert!(!state.wants_pointer_capture());
        let regrabbed = state.handle_click(&bar, r, 20, 5);
        assert_eq!(
            regrabbed,
            Some(GizmoClickOutcome::ResizeEdgeGrabbed(ResizeEdge::Right))
        );
    }

    #[test]
    fn seamless_toggles_immediately_without_starting_a_drag() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();

        state.handle_click(&bar, r, 7, 9);

        assert!(state.is_seamless());
        assert!(!state.wants_pointer_capture());
    }

    #[test]
    fn hovered_gizmo_uses_the_same_vivid_treatment_as_an_active_one() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let palette = UiPalette::default();
        let mut state = GizmoState::new();
        state.note_pointer(&bar, r, 1, 9);

        assert_eq!(
            state.gizmo_color(GizmoKind::Move, &palette),
            palette.get(UiColorRole::Vivid)
        );
        assert_eq!(state.gizmo_weight_index(GizmoKind::Move), 3);
        assert_eq!(
            state.gizmo_color(GizmoKind::Resize, &palette),
            palette.get(UiColorRole::Medium)
        );
    }

    #[test]
    fn move_mode_makes_every_border_vivid_and_heavier() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let palette = UiPalette::default();
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 1, 9);

        for edge in [
            ResizeEdge::Left,
            ResizeEdge::Right,
            ResizeEdge::Top,
            ResizeEdge::Bottom,
        ] {
            assert_eq!(
                state.border_color(edge, &palette),
                palette.get(UiColorRole::Vivid)
            );
            assert_eq!(state.border_weight_index(edge), 2);
        }
    }

    #[test]
    fn resize_hover_emphasizes_only_the_hovered_edge() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let palette = UiPalette::default();
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 5, 9);
        state.note_pointer(&bar, r, 20, 5);

        assert_eq!(
            state.border_color(ResizeEdge::Right, &palette),
            palette.get(UiColorRole::Vivid)
        );
        assert_eq!(state.border_weight_index(ResizeEdge::Right), 3);
        assert_eq!(
            state.border_color(ResizeEdge::Left, &palette),
            palette.get(UiColorRole::Medium)
        );
        assert_eq!(state.border_weight_index(ResizeEdge::Left), 2);
    }

    #[test]
    fn should_draw_gizmo_bar_stays_true_when_not_seamless_regardless_of_hover() {
        let state = GizmoState::new();
        assert!(state.should_draw_gizmo_bar());
    }

    #[test]
    fn seamless_hides_the_gizmo_bar_until_hovered() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();
        state.handle_click(&bar, r, 7, 9); // toggle seamless on

        assert!(!state.should_draw_gizmo_bar());

        state.set_hovered(true);
        assert!(state.should_draw_gizmo_bar());

        state.set_hovered(false);
        assert!(!state.should_draw_gizmo_bar());
    }

    #[test]
    fn close_is_reported_to_the_caller_but_does_not_change_gizmo_state() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();

        let clicked = state.handle_click(&bar, r, 3, 9);

        assert_eq!(clicked, Some(GizmoClickOutcome::Gizmo(GizmoKind::Close)));
        assert!(!state.wants_pointer_capture());
    }

    #[test]
    fn a_click_outside_the_gizmo_bar_is_ignored() {
        let bar = full_bar();
        let r = rect(0, 0, 20, 10);
        let mut state = GizmoState::new();

        assert_eq!(state.handle_click(&bar, r, 5, 5), None);
    }

    #[test]
    fn hotspots_with_keeps_gizmo_hotspots_first_then_custom() {
        let bar = GizmoBar::new(vec![GizmoKind::Move]);
        let r = rect(0, 0, 20, 10);
        let custom = vec![Hotspot::new(
            ModuleRect { x0: 3, y0: 3, x1: 3, y1: 3 },
            "row",
            "custom control",
        )];
        let hotspots = bar.hotspots_with(r, custom);
        assert_eq!(hotspots.len(), 2);
        assert_eq!(hotspots[0].title, "move");
        assert_eq!(hotspots[1].title, "row");
        // No custom list: identical to hotspots().
        assert_eq!(bar.hotspots_with(r, Vec::new()), bar.hotspots(r));
    }
}
