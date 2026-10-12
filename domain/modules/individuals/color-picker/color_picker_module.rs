use crate::{
    canonical_sprite_palette, title_hotspot, Cell, CellColor, CellGraphic, CellGroup,
    CellGroupIntakeBehavior, CellPoint, GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState,
    Hotspot, Module, ModulePointerEvent, ModuleRect, PanelChrome, PersistedModuleUiState,
    UiColorRole, UiPalette, WorldPoint,
};

/// `(columns, content_height)`: how many swatches fit across the content
/// width currently available inside `rect` before wrapping to the next row
/// (packed edge-to-edge, no gap), and how many content rows `rect` still
/// offers the swatch grid after the footer reserve — this is what makes the
/// list reflow on a live resize instead of assuming a fixed column count,
/// and what both `draw()` and a click's hit-test read to line up with
/// whatever the current size just drew. `columns` is always at least `1`,
/// even for a `rect` too narrow to hold a full border.
fn swatch_layout(rect: ModuleRect) -> (i32, i32) {
    let (content_width, content_height) = PanelChrome::content_size(rect);
    (content_width.max(1), content_height)
}

/// `(value, hue, saturation)` of one RGB color in 0..=1 / 0..360 / 0..=1
/// terms — the inputs of the display-order sort key. Value is the max
/// channel; hue is the standard HSV hue; saturation is delta/value (black
/// has no chroma and no meaningful hue, so it reads as saturation 0).
fn value_hue_saturation(rgb: [u8; 3]) -> (f32, f32, f32) {
    let channel = |index: usize| rgb[index] as f32 / 255.0;
    let (r, g, b) = (channel(0), channel(1), channel(2));
    let value = r.max(g).max(b);
    let delta = value - r.min(g).min(b);
    let saturation = if value <= 0.0 { 0.0 } else { delta / value };
    let hue = if delta <= 0.0 {
        0.0
    } else if value == r {
        ((g - b) / delta * 60.0).rem_euclid(360.0)
    } else if value == g {
        ((b - r) / delta * 60.0 + 120.0).rem_euclid(360.0)
    } else {
        ((r - g) / delta * 60.0 + 240.0).rem_euclid(360.0)
    };
    (value, hue, saturation)
}

/// The gray-band cutoff: colors at or below this saturation sort as the
/// gray ramp instead of a hue family, so near-neutrals don't strand as
/// single-color "hue families" at the far end of the wheel.
const GRAY_SATURATION_MAX: f32 = 0.05;

/// The display sort key for one color. Grays (saturation at or below the
/// buffer zone) sort as their own dark→light value ramp ahead of every
/// chromatic color. Chromatic colors sort hue-major — each hue family is
/// one light→dark value ramp (saturation descending as the tie-break) —
/// so "a darker version of this red" is always the next cell, and the
/// panel reads as a palette breakdown: how many hue families, and whether
/// a ramp is missing steps.
fn color_sort_key(rgb: [u8; 3]) -> (u8, f32, f32, f32) {
    let (value, hue, saturation) = value_hue_saturation(rgb);
    if saturation <= GRAY_SATURATION_MAX {
        (0, value, 0.0, 0.0)
    } else {
        (1, hue, -value, -saturation)
    }
}

/// Where order position `order_index` (0 = top-left) lands in the flat
/// swatch list, given how many `columns` currently fit and how tall the
/// content area is — order index 0 is nearest the top border, so the list
/// aggregates downward from the top like an ordinary top-to-bottom list,
/// wrapping to the right within a row.
fn swatch_position(order_index: usize, columns: i32, content_height: i32) -> (i32, i32) {
    let order_index = order_index as i32;
    let row = order_index / columns;
    (order_index % columns, content_height - 1 - row)
}

/// A flowing-list color picker drawing every color in the renderer's
/// canonical palette as one clickable swatch, packed edge-to-edge with no
/// gap and wrapping to fit whatever width is currently available. Display
/// order is derived from the colors themselves — the gray ramp first
/// (dark→light), then hue families each running light→dark — so the list
/// never reorganizes on click; clicking only
/// selects. Swatches that do not fit the current content size are simply
/// not drawn or hittable until the panel is grown. Generic enough for any
/// thaum-renderer consumer, so it lives here rather than in a per-program
/// `individuals/`. Draws itself with the shared move/close/resize/seamless
/// gizmo bar.
pub struct ColorPickerModule {
    id: String,
    rect: ModuleRect,
    palette: UiPalette,
    indexed_palette: Vec<[u8; 3]>,
    /// The selected swatch's raw RGB, keyed by color rather than list
    /// position so a palette swap or re-sort never loses the selection.
    selected_rgb: Option<[u8; 3]>,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    /// Bottom content rows a wrapper reserves for its own footer strip: the
    /// swatch grid lays out and hit-tests only the content rows above them,
    /// so wrapper-drawn footer content never shares a row with a swatch.
    footer_rows: i32,
    hidden: bool,
}

impl ColorPickerModule {
    pub fn new(id: impl Into<String>, rect: ModuleRect, palette: UiPalette) -> Self {
        let indexed_palette = canonical_sprite_palette().to_vec();
        Self {
            id: id.into(),
            rect,
            palette,
            selected_rgb: None,
            indexed_palette,
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            footer_rows: 0,
            hidden: false,
        }
    }

    /// Reserves `footer_rows` bottom content rows for a wrapper's own
    /// footer strip: the swatch grid shrinks to the content rows above
    /// them, so the panel keeps one chrome (one bottom border) while the
    /// wrapper renders its strip inside the content area's bottom.

    pub fn with_indexed_palette(mut self, indexed_palette: &[[u8; 3]]) -> Self {
        self.set_indexed_palette(indexed_palette);
        self
    }

    pub fn with_footer_rows(mut self, footer_rows: i32) -> Self {
        self.footer_rows = footer_rows;
        self
    }

    /// Replaces the swatch palette at runtime. The consumer owns the palette
    /// policy and may swap the live list (for example a document-owned
    /// palette the user edits); the selection survives the swap when its
    /// color is still in the new list and drops when it is not. Display
    /// order is always derived from the colors, never stored.
    pub fn set_indexed_palette(&mut self, indexed_palette: &[[u8; 3]]) {
        self.indexed_palette = if indexed_palette.is_empty() {
            canonical_sprite_palette().to_vec()
        } else {
            indexed_palette.to_vec()
        };
        if let Some(rgb) = self.selected_rgb {
            if !self.indexed_palette.contains(&rgb) {
                self.selected_rgb = None;
            }
        }
    }

    /// The currently selected swatch's raw RGB, if any swatch has been clicked yet.
    pub fn selected_rgb(&self) -> Option<[u8; 3]> {
        self.selected_rgb
    }

    /// The display order as palette indices — the gray ramp (dark→light)
    /// first, then hue families each running light→dark — a read view for
    /// consumers and tests mapping a display position back to a color;
    /// `draw()`, the
    /// hotspots, and click hit-testing all read this same order.
    pub fn display_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.indexed_palette.len()).collect();
        order.sort_by(|&a, &b| {
            // f32 has no `Ord`, so the key compares field by field.
            let key_a = color_sort_key(self.indexed_palette[a]);
            let key_b = color_sort_key(self.indexed_palette[b]);
            key_a
                .0
                .cmp(&key_b.0)
                .then_with(|| key_a.1.total_cmp(&key_b.1))
                .then_with(|| key_a.2.total_cmp(&key_b.2))
                .then_with(|| key_a.3.total_cmp(&key_b.3))
        });
        order
    }

    /// Replaces the panel rect directly — for wrappers that reserve part of
    /// their own bounds around this picker.
    pub fn set_rect(&mut self, rect: ModuleRect) {
        self.rect = rect;
    }

    /// Selects the color now displayed at `order_index`, clamped to the
    /// list — the repeated-`−` wipe seam: after a wrapper removes the
    /// selected swatch, the color that took its display slot becomes the
    /// selection, so pressing `−` again removes the next one. An empty
    /// palette clears the selection.
    pub fn select_display_position(&mut self, order_index: usize) {
        let count = self.indexed_palette.len();        if count == 0 {
            self.selected_rgb = None;
            return;
        }
        let order_index = order_index.min(count - 1);
        let index = self.display_order()[order_index];
        self.selected_rgb = Some(self.indexed_palette[index]);
    }

    /// Whether seamless mode currently strips the border/background/title.
    pub fn is_seamless(&self) -> bool {
        self.gizmo_state.is_seamless()
    }

    pub fn set_seamless(&mut self, seamless: bool) {
        self.gizmo_state.set_seamless(seamless);
    }

    /// The palette currently driving the swatch list (read view for
    /// consumers asserting or syncing against it).
    pub fn indexed_palette(&self) -> &[[u8; 3]] {
        &self.indexed_palette
    }
}

impl Module for ColorPickerModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    /// Tooltip hotspots: the module's gizmo bar plus one hotspot per drawn
    /// swatch, so every color explains itself through the shared tooltip
    /// implementation.
    fn hotspots(&self) -> Vec<Hotspot> {
        let (content_x, content_y) = PanelChrome::content_origin();
        let (columns, content_height) = swatch_layout(self.rect);
        let capacity = (columns.max(0) * (content_height - self.footer_rows).max(0)) as usize;
        let mut custom = vec![title_hotspot(
            self.rect,
            self.gizmos.title_start_x(),
            "color module",
            "picks a palette color for your given hand. left click for left hand, right click for right hand.",
        )];
        custom.extend(
            self.display_order()
                .into_iter()
                .take(capacity)
                .enumerate()
                .map(|(order_index, index)| {
                    let (column, row) = swatch_position(order_index, columns, content_height);
                    let x = self.rect.x0 + content_x + column;
                    let y = self.rect.y0 + content_y + row;
                    let rgb = self.indexed_palette[index];
                    Hotspot::new(
                        ModuleRect {
                            x0: x,
                            y0: y,
                            x1: x,
                            y1: y,
                        },
                        "color swatch",
                        format!("click to use rgb {},{},{}", rgb[0], rgb[1], rgb[2]),
                    )
                }),
        );
        self.gizmos.hotspots_with(self.rect, custom)
    }

    fn draw(&self) -> CellGroup {
        let origin = WorldPoint {
            x: self.rect.x0,
            y: self.rect.y0,
            z: 0,
        };
        let (content_x, content_y) = PanelChrome::content_origin();
        let (columns, content_height) = swatch_layout(self.rect);
        let swatches = &self.indexed_palette;
        let vivid = self.palette.get(UiColorRole::Vivid);

        let mut cells: Vec<Cell> = if self.gizmo_state.is_seamless() {
            Vec::new()
        } else {
            self.gizmo_state
                .decorate_panel_chrome(
                    PanelChrome::new(self.rect, &self.palette)
                        .with_title("COLORS")
                        .with_title_start_x(self.gizmos.title_start_x()),
                    &self.palette,
                )
                .cells()
        };
        if self.gizmo_state.should_draw_gizmo_bar() {
            cells.extend(
                self.gizmos
                    .cells(self.rect, &self.gizmo_state, &self.palette),
            );
        }

        let capacity = (columns.max(0) * content_height.max(0)) as usize;
        cells.extend(
            self.display_order()
                .into_iter()
                .take(capacity)
                .enumerate()
                .map(|(order_index, index)| {
                    let (column, row) = swatch_position(order_index, columns, content_height);
                    let rgb = swatches[index];
                    let is_selected = self.selected_rgb == Some(rgb);
                    Cell {
                        position: CellPoint {
                            x: content_x + column,
                            y: content_y + row,
                            z: 0,
                        },
                        graphic: CellGraphic::Glyph(if is_selected { '◆' } else { '█' }),
                        color: if is_selected {
                            vivid.clone()
                        } else {
                            CellColor::Flat([
                                rgb[0] as f32 / 255.0,
                                rgb[1] as f32 / 255.0,
                                rgb[2] as f32 / 255.0,
                                1.0,
                            ])
                        },
                        ..Cell::default()
                    }
                }),
        );

        CellGroup::from_cells(origin, cells).with_intake_behavior(CellGroupIntakeBehavior::Flat2d)
    }

    fn on_pointer_event(&mut self, event: ModulePointerEvent) {
        match event {
            ModulePointerEvent::Click { x, y, .. } => {
                if let Some(outcome) = self.gizmo_state.handle_click(&self.gizmos, self.rect, x, y)
                {
                    if outcome == GizmoClickOutcome::Gizmo(GizmoKind::Close) {
                        self.hidden = true;
                    }
                    return;
                }

                let (content_x, content_y) = PanelChrome::content_origin();
                let (columns, content_height) = swatch_layout(self.rect);
                let local_x = x - self.rect.x0 - content_x;
                let local_y = y - self.rect.y0 - content_y;
                if local_x < 0
                    || local_y < 0
                    || local_x >= columns
                    || local_y >= content_height
                    || local_y < self.footer_rows
                {
                    return;
                }

                let row = content_height - 1 - local_y;
                let order_index = (row * columns + local_x) as usize;
                let capacity = (columns.max(0) * content_height.max(0)) as usize;
                if order_index >= capacity || order_index >= self.indexed_palette.len() {
                    return;
                }

                let index = self.display_order()[order_index];
                self.selected_rgb = Some(self.indexed_palette[index]);
            }
            ModulePointerEvent::Move { x, y } => {
                self.gizmo_state.note_pointer(&self.gizmos, self.rect, x, y);
                if let Some(next_rect) = self.gizmo_state.drag_rect(x, y) {
                    self.rect = next_rect;
                }
            }
            ModulePointerEvent::Up { .. } => {
                self.gizmo_state.end_drag();
            }
            ModulePointerEvent::Enter => {
                self.gizmo_state.set_hovered(true);
            }
            ModulePointerEvent::Leave => {
                self.gizmo_state.set_hovered(false);
            }
            ModulePointerEvent::Down { .. } => {}
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.gizmo_state.wants_pointer_capture()
    }

    fn is_hidden(&self) -> bool {
        self.hidden
    }

    fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    fn persisted_ui_state(&self) -> Option<PersistedModuleUiState> {
        Some(PersistedModuleUiState::new(
            self.id(),
            self.rect,
            self.gizmo_state.is_seamless(),
            self.hidden,
        ))
    }

    fn apply_persisted_ui_state(&mut self, state: &PersistedModuleUiState) {
        self.rect = state.rect.to_runtime();
        self.gizmo_state.set_seamless(state.is_seamless);
        self.hidden = state.is_hidden;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModulePointerButton;

    // Wide/tall enough to fit all 24 swatches (14 columns x 2 rows, packed
    // edge-to-edge) plus a 1-cell chrome inset on every side, with the
    // old-style header row reserved inside the border for the full gizmo
    // bar (four glyphs, two cells each) followed by the "COLORS" title.
    fn rect(x0: i32, y0: i32) -> ModuleRect {
        ModuleRect {
            x0,
            y0,
            x1: x0 + 16,
            y1: y0 + 8,
        }
    }

    fn swatch_glyph_cells(group: &CellGroup) -> usize {
        group
            .cells
            .values()
            .filter(|cell| {
                matches!(
                    cell.graphic,
                    CellGraphic::Glyph('█') | CellGraphic::Glyph('◆')
                )
            })
            .count()
    }

    #[test]
    fn draws_one_swatch_cell_per_canonical_palette_color() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        assert_eq!(swatch_glyph_cells(&group), canonical_sprite_palette().len());
    }

    #[test]
    fn draws_a_chrome_border_gizmo_bar_and_title_around_the_grid() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        assert_eq!(
            group.cells[&CellPoint { x: 0, y: 0, z: 0 }].graphic,
            CellGraphic::Glyph('┗')
        );
        // Gizmo bar: move, close, resize, seamless at local x = 1, 3, 5, 7.
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 7, z: 0 }].graphic,
            CellGraphic::Glyph('#')
        );
        assert_eq!(
            group.cells[&CellPoint { x: 7, y: 7, z: 0 }].graphic,
            CellGraphic::Glyph('S')
        );
        // Title starts right after the gizmo bar (local x = 9) on the header row.
        assert_eq!(
            group.cells[&CellPoint { x: 9, y: 7, z: 0 }].graphic,
            CellGraphic::Glyph('C')
        );
    }

    #[test]
    fn no_swatch_is_selected_before_any_click() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        assert_eq!(picker.selected_rgb(), None);
    }

    #[test]
    fn clicking_a_swatch_selects_its_exact_palette_color() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        // display order position 5 (row 0 nearest the top, column 5, packed
        // with no gap) -> local (1 + 5, 1 + (content_height - 1)) = (6, 5)
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        });

        let order = picker.display_order();
        let expected = canonical_sprite_palette()[order[5]];
        assert_eq!(picker.selected_rgb(), Some(expected));
    }

    #[test]
    fn click_outside_the_grid_is_ignored() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 100,
            y: 100,
            button: ModulePointerButton::Left,
        });

        assert_eq!(picker.selected_rgb(), None);
    }

    #[test]
    fn click_on_the_chrome_border_itself_is_ignored() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 0,
            y: 0,
            button: ModulePointerButton::Left,
        });

        assert_eq!(picker.selected_rgb(), None);
    }

    #[test]
    fn selected_swatch_draws_with_the_vivid_role_color() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        // display order position 0 (top-left) -> local (1, 1 + (content_height - 1)) = (1, 5)
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 5,
            button: ModulePointerButton::Left,
        });

        let group = picker.draw();
        let selected_cell = &group.cells[&CellPoint { x: 1, y: 5, z: 0 }];
        assert_eq!(selected_cell.color, picker.palette.get(UiColorRole::Vivid));
        assert_eq!(selected_cell.graphic, CellGraphic::Glyph('◆'));
    }

    #[test]
    fn clicking_the_move_gizmo_starts_requesting_pointer_capture() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 7,
            button: ModulePointerButton::Left,
        });

        assert!(picker.wants_pointer_capture());
    }

    #[test]
    fn dragging_after_a_move_click_relocates_the_panel_in_real_time() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 7,
            button: ModulePointerButton::Left,
        });
        picker.on_pointer_event(ModulePointerEvent::Move { x: 4, y: 10 });

        assert_eq!(picker.rect(), rect(3, 3));
    }

    #[test]
    fn releasing_the_pointer_ends_the_drag_and_stops_capture_requests() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 7,
            button: ModulePointerButton::Left,
        });
        picker.on_pointer_event(ModulePointerEvent::Move { x: 4, y: 10 });
        picker.on_pointer_event(ModulePointerEvent::Up { x: 4, y: 10 });

        assert!(!picker.wants_pointer_capture());
        // A further move without a new gizmo click no longer moves the panel.
        picker.on_pointer_event(ModulePointerEvent::Move { x: 40, y: 40 });
        assert_eq!(picker.rect(), rect(3, 3));
    }

    #[test]
    fn clicking_resize_then_grabbing_an_edge_and_dragging_grows_only_that_edge() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 5,
            y: 7,
            button: ModulePointerButton::Left,
        }); // arm resize
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 16,
            y: 4,
            button: ModulePointerButton::Left,
        }); // grab right edge
        picker.on_pointer_event(ModulePointerEvent::Move { x: 20, y: 4 });

        assert_eq!(
            picker.rect(),
            ModuleRect {
                x0: 0,
                y0: 0,
                x1: 20,
                y1: 8
            }
        );
    }

    #[test]
    fn a_plain_click_inside_the_panel_while_resize_is_armed_still_selects_a_swatch() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 5,
            y: 7,
            button: ModulePointerButton::Left,
        }); // arm resize
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        }); // swatch, not an edge

        let order = picker.display_order();
        let expected = canonical_sprite_palette()[order[5]];
        assert_eq!(picker.selected_rgb(), Some(expected));
    }

    #[test]
    fn clicking_the_close_gizmo_marks_the_panel_closed() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 3,
            y: 7,
            button: ModulePointerButton::Left,
        });

        assert!(picker.is_hidden());
    }

    #[test]
    fn clicking_the_seamless_gizmo_hides_the_border_and_the_gizmo_bar_until_hovered() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 7,
            y: 7,
            button: ModulePointerButton::Left,
        });

        let group = picker.draw();
        assert!(!group.cells.contains_key(&CellPoint { x: 0, y: 0, z: 0 }));
        assert!(!has_gizmo_bar(&group));

        picker.on_pointer_event(ModulePointerEvent::Enter);
        let hovered_group = picker.draw();
        assert_eq!(
            hovered_group.cells[&CellPoint { x: 7, y: 7, z: 0 }].graphic,
            CellGraphic::Glyph('S')
        );
    }

    #[test]
    fn a_gizmo_click_does_not_also_select_a_swatch() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 7,
            button: ModulePointerButton::Left,
        });

        assert_eq!(picker.selected_rgb(), None);
    }

    fn has_gizmo_bar(group: &CellGroup) -> bool {
        group.cells.contains_key(&CellPoint { x: 1, y: 7, z: 0 })
    }

    #[test]
    fn seamless_hides_the_gizmo_bar_until_the_pointer_enters_and_hides_it_again_on_leave() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 7,
            y: 7,
            button: ModulePointerButton::Left,
        }); // toggle seamless

        assert!(!has_gizmo_bar(&picker.draw()));

        picker.on_pointer_event(ModulePointerEvent::Enter);
        assert!(has_gizmo_bar(&picker.draw()));

        picker.on_pointer_event(ModulePointerEvent::Leave);
        assert!(!has_gizmo_bar(&picker.draw()));
    }

    #[test]
    fn hovering_does_not_affect_the_gizmo_bar_when_not_seamless() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());

        assert!(has_gizmo_bar(&picker.draw()));
        picker.on_pointer_event(ModulePointerEvent::Leave);
        assert!(has_gizmo_bar(&picker.draw()));
    }

    #[test]
    fn shrinking_the_panel_below_default_reflows_swatches_into_more_rows_and_still_fits_all_of_them(
    ) {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 5,
            y: 7,
            button: ModulePointerButton::Left,
        }); // arm resize
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 16,
            y: 4,
            button: ModulePointerButton::Left,
        }); // grab right edge
        picker.on_pointer_event(ModulePointerEvent::Move { x: 8, y: 4 }); // shrink hard

        let group = picker.draw();
        assert_eq!(swatch_glyph_cells(&group), canonical_sprite_palette().len());
    }

    #[test]
    fn swatches_that_do_not_fit_the_shrunk_content_are_clipped_not_drawn_past_the_border() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 5,
            y: 7,
            button: ModulePointerButton::Left,
        }); // arm resize
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 16,
            y: 4,
            button: ModulePointerButton::Left,
        }); // grab right edge
        // width 6 -> 4 content columns x 5 content rows = 20 capacity < 24
        picker.on_pointer_event(ModulePointerEvent::Move { x: 6, y: 4 });

        let group = picker.draw();
        assert_eq!(swatch_glyph_cells(&group), 20);
    }

    #[test]
    fn swatches_pack_edge_to_edge_with_no_gap_and_wrap_at_the_content_width() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        // content width is 14 cells and content height is 5 (16x8 rect minus
        // a 1-cell inset on each side plus the reserved header row), so display
        // positions 0..13 fill row 0 — nearest the top content row, local y = 5 —
        // back-to-back with no gap between adjacent swatches, then position 14 wraps
        // to the row below (local y = 4).
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 5, z: 0 }].graphic,
            CellGraphic::Glyph('█')
        );
        assert_eq!(
            group.cells[&CellPoint { x: 14, y: 5, z: 0 }].graphic,
            CellGraphic::Glyph('█')
        );
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 4, z: 0 }].graphic,
            CellGraphic::Glyph('█')
        );
    }

    #[test]
    fn footer_rows_keep_the_swatch_grid_out_of_the_reserved_bottom_rows() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_footer_rows(2);
        let group = picker.draw();

        // content is 14x5; a 2-row footer reserve shrinks the grid to 3
        // rows, so the bottom two content rows (local y 1 and 2) hold no
        // swatches while the top row still packs from local y = 5.
        for y in [1, 2] {
            for x in 1..=14 {
                assert_ne!(
                    group.cells[&CellPoint { x, y, z: 0 }].graphic,
                    CellGraphic::Glyph('█'),
                    "a swatch leaked into the reserved footer row {y}"
                );
            }
        }
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 5, z: 0 }].graphic,
            CellGraphic::Glyph('█')
        );
    }

    #[test]
    fn a_click_in_a_reserved_footer_row_does_not_select_a_swatch() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_footer_rows(2);

        // local (6, 2) is inside the reserved footer rows.
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 2,
            button: ModulePointerButton::Left,
        });
        assert_eq!(picker.selected_rgb(), None);

        // a real swatch click still selects.
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        });
        assert_eq!(
            picker.selected_rgb(),
            Some(canonical_sprite_palette()[picker.display_order()[5]])
        );
    }

    #[test]
    fn clicking_a_swatch_reorders_nothing() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        // display order position 5 (row 0, column 5) -> local (1 + 5, 1 + 4) = (6, 5)
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        });

        let order = picker.display_order();
        let group = picker.draw();
        let selected_cell = &group.cells[&CellPoint { x: 6, y: 5, z: 0 }];
        assert_eq!(selected_cell.graphic, CellGraphic::Glyph('◆'));
        assert_eq!(selected_cell.color, picker.palette.get(UiColorRole::Vivid));

        // the slot right of the selection still holds exactly the color
        // display order puts there — the click selected, it did not move
        // anything in the list.
        let neighbor_rgb = canonical_sprite_palette()[order[6]];
        assert_eq!(
            group.cells[&CellPoint { x: 7, y: 5, z: 0 }].color,
            CellColor::Flat([
                neighbor_rgb[0] as f32 / 255.0,
                neighbor_rgb[1] as f32 / 255.0,
                neighbor_rgb[2] as f32 / 255.0,
                1.0,
            ])
        );
    }

    #[test]
    fn display_order_sorts_the_gray_ramp_first_then_hue_major_value_ramps() {
        // grays (including the 5%-saturation buffer zone) form a dark→light
        // ramp first; chromatic colors then sort hue-major with a light→dark
        // ramp inside each hue family.
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_indexed_palette(&[
                [255, 255, 255], // white — gray ramp end
                [0, 255, 255],   // cyan — hue 180
                [128, 0, 0],     // dark red — hue 0, value 0.5
                [0, 0, 0],       // black — gray ramp start
                [255, 0, 0],     // bright red — hue 0, value 1.0
                [250, 240, 240], // near-gray — saturation 0.04, inside the buffer zone
                [128, 128, 128], // mid gray
            ]);

        let order = picker.display_order();
        let palette = picker.indexed_palette();
        assert_eq!(palette[order[0]], [0, 0, 0]);
        assert_eq!(palette[order[1]], [128, 128, 128]);
        assert_eq!(palette[order[2]], [250, 240, 240]);
        assert_eq!(palette[order[3]], [255, 255, 255]);
        // hue 0 family: value descending, so bright red before dark red.
        assert_eq!(palette[order[4]], [255, 0, 0]);
        assert_eq!(palette[order[5]], [128, 0, 0]);
        // hue 180 family last.
        assert_eq!(palette[order[6]], [0, 255, 255]);
    }

    #[test]
    fn same_hue_and_value_sorts_more_saturated_first() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_indexed_palette(&[
                [255, 64, 64], // hue 0, value 1.0, saturation 0.75
                [255, 0, 0],   // hue 0, value 1.0, saturation 1.0
            ]);

        let order = picker.display_order();
        assert_eq!(picker.indexed_palette()[order[0]], [255, 0, 0]);
        assert_eq!(picker.indexed_palette()[order[1]], [255, 64, 64]);
    }

    #[test]
    fn selection_survives_a_palette_swap_when_the_color_stays_and_drops_when_it_leaves() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_indexed_palette(&[[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 1,
            y: 5,
            button: ModulePointerButton::Left,
        });
        let selected = picker.selected_rgb().expect("a swatch was clicked");

        picker.set_indexed_palette(&[[9, 9, 9], selected]);
        assert_eq!(picker.selected_rgb(), Some(selected));

        picker.set_indexed_palette(&[[9, 9, 9], [8, 8, 8]]);
        assert_eq!(picker.selected_rgb(), None);
    }

    #[test]
    fn select_display_position_picks_the_clamped_slot() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_indexed_palette(&[[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        picker.select_display_position(1);
        let order = picker.display_order();
        assert_eq!(picker.selected_rgb(), Some(picker.indexed_palette()[order[1]]));

        // Past the end clamps to the last slot. (The picker never holds an
        // empty palette: `set_indexed_palette(&[])` re-seeds the canonical
        // default, matching the painter's own non-empty palette policy.)
        picker.select_display_position(99);
        let order = picker.display_order();
        assert_eq!(picker.selected_rgb(), Some(picker.indexed_palette()[order[2]]));
    }

    #[test]
    fn the_swatch_list_aggregates_downward_from_the_top_not_upward_from_the_bottom() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        // display position 0 sits at the highest content row (local y = 5, below
        // the header row at local y = 7), not the lowest — the lowest content
        // row (local y = 1) is still plain chrome background,
        // not a swatch.
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 5, z: 0 }].graphic,
            CellGraphic::Glyph('█')
        );
        assert_eq!(
            group.cells[&CellPoint { x: 1, y: 1, z: 0 }].graphic,
            CellGraphic::Glyph(' ')
        );
    }

    #[test]
    fn custom_indexed_palette_replaces_the_default_swatches() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default())
            .with_indexed_palette(&[[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
        let group = picker.draw();

        assert_eq!(swatch_glyph_cells(&group), 3);
    }
}
