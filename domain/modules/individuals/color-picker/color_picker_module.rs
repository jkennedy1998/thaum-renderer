use crate::{
    canonical_sprite_palette, Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior,
    CellPoint, GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, Hotspot, Module,
    ModulePointerEvent, ModuleRect, PanelChrome, PersistedModuleUiState, UiColorRole, UiPalette,
    WorldPoint,
};

/// `(columns, content_height)`: how many swatches fit across the content
/// width currently available inside `rect` before wrapping to the next row
/// (packed edge-to-edge, no gap), and how many content rows tall `rect`
/// currently is — this is what makes the list reflow on a live resize
/// instead of assuming a fixed column count, and what both `draw()` and a
/// click's hit-test read to line up with whatever the current size just
/// drew. `columns` is always at least `1`, even for a `rect` too narrow to
/// hold a full border.
fn swatch_layout(rect: ModuleRect) -> (i32, i32) {
    let (content_width, content_height) = PanelChrome::content_size(rect);
    (content_width.max(1), content_height)
}

/// Where order position `order_index` (0 = most recently clicked, or
/// palette order before any click) lands in the flat swatch list, given how
/// many `columns` currently fit and how tall the content area is —
/// `order_index` 0 is nearest the top border, so the list aggregates
/// downward from the top like an ordinary top-to-bottom list, wrapping to
/// the right within a row and overflowing past the bottom border once
/// there are more swatches than fit.
fn swatch_position(order_index: usize, columns: i32, content_height: i32) -> (i32, i32) {
    let order_index = order_index as i32;
    let row = order_index / columns;
    (order_index % columns, content_height - 1 - row)
}

/// A flowing-list color picker drawing every color in the renderer's
/// canonical palette as one clickable swatch, packed edge-to-edge with no
/// gap and wrapping to fit whatever width is currently available. Clicking
/// a swatch both selects it and reorganizes the list, moving it to the
/// front — so the picker doubles as a most-recently-used order. Generic
/// enough for any thaum-renderer consumer, so it lives here rather than in
/// a per-program `individuals/`. Draws itself with the shared move/close/
/// resize/seamless gizmo bar, so it is also the first real proof that
/// `domain/modules/shared/module-gizmos/` works end to end.
pub struct ColorPickerModule {
    id: String,
    rect: ModuleRect,
    palette: UiPalette,
    indexed_palette: Vec<[u8; 3]>,
    /// Palette indices in current display order — index 0 is drawn first
    /// (top-left), and is always the most recently clicked swatch once
    /// anything has been clicked.
    order: Vec<usize>,
    selected_index: Option<usize>,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
}

impl ColorPickerModule {
    pub fn new(id: impl Into<String>, rect: ModuleRect, palette: UiPalette) -> Self {
        let indexed_palette = canonical_sprite_palette().to_vec();
        Self {
            id: id.into(),
            rect,
            palette,
            order: (0..indexed_palette.len()).collect(),
            indexed_palette,
            selected_index: None,
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            hidden: false,
        }
    }

    pub fn with_indexed_palette(mut self, indexed_palette: &[[u8; 3]]) -> Self {
        self.indexed_palette = if indexed_palette.is_empty() {
            canonical_sprite_palette().to_vec()
        } else {
            indexed_palette.to_vec()
        };
        self.order = (0..self.indexed_palette.len()).collect();
        self.selected_index = None;
        self
    }

    /// The currently selected swatch's raw RGB, if any swatch has been clicked yet.
    pub fn selected_rgb(&self) -> Option<[u8; 3]> {
        self.selected_index.map(|index| self.indexed_palette[index])
    }
}

impl Module for ColorPickerModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    /// Tooltip hotspots: the module's gizmo bar, so every gizmo-enabled
    /// panel grows tooltips from one shared implementation.
    fn hotspots(&self) -> Vec<Hotspot> {
        self.gizmos.hotspots(self.rect)
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

        cells.extend(self.order.iter().enumerate().map(|(order_index, &index)| {
            let (column, row) = swatch_position(order_index, columns, content_height);
            let rgb = swatches[index];
            let is_selected = self.selected_index == Some(index);
            Cell {
                position: CellPoint {
                    x: content_x + column,
                    y: content_y + row,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(if is_selected { '◆' } else { '█' }),
                color: if is_selected {
                    vivid
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
        }));

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
                if local_x < 0 || local_y < 0 || local_x >= columns || local_y >= content_height {
                    return;
                }

                let row = content_height - 1 - local_y;
                let order_index = (row * columns + local_x) as usize;
                if order_index >= self.order.len() {
                    return;
                }

                let index = self.order.remove(order_index);
                self.order.insert(0, index);
                self.selected_index = Some(index);
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
        // order index 5 (row 0 nearest the top, column 5, packed with no gap)
        // -> local (1 + 5, 1 + (content_height - 1)) = (6, 1 + 4) = (6, 5)
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        });

        let expected = canonical_sprite_palette()[5];
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
        // order index 0 (top-left) -> local (1, 1 + (content_height - 1)) = (1, 5)
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

        let expected = canonical_sprite_palette()[5];
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
    fn swatches_pack_edge_to_edge_with_no_gap_and_wrap_at_the_content_width() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        // content width is 14 cells and content height is 5 (16x8 rect minus
        // a 1-cell inset on each side plus the reserved header row), so order
        // indices 0..13 fill row 0 — nearest the top content row, local y = 5 —
        // back-to-back with no gap between adjacent swatches, then index 14 wraps
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
    fn clicking_a_swatch_reorganizes_it_to_the_front_of_the_list() {
        let mut picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        // order index 5 (row 0, column 5) -> local (1 + 5, 1 + 4) = (6, 5)
        picker.on_pointer_event(ModulePointerEvent::Click {
            x: 6,
            y: 5,
            button: ModulePointerButton::Left,
        });

        let group = picker.draw();
        let front_cell = &group.cells[&CellPoint { x: 1, y: 5, z: 0 }];
        assert_eq!(front_cell.graphic, CellGraphic::Glyph('◆'));
        assert_eq!(front_cell.color, picker.palette.get(UiColorRole::Vivid));

        // the swatch that used to be first (palette index 0) is now bumped
        // one slot back, proving the click reorganized the list rather than
        // just relabeling the front slot as selected.
        let bumped_rgb = canonical_sprite_palette()[0];
        assert_eq!(
            group.cells[&CellPoint { x: 2, y: 5, z: 0 }].color,
            CellColor::Flat([
                bumped_rgb[0] as f32 / 255.0,
                bumped_rgb[1] as f32 / 255.0,
                bumped_rgb[2] as f32 / 255.0,
                1.0,
            ])
        );
    }

    #[test]
    fn the_swatch_list_aggregates_downward_from_the_top_not_upward_from_the_bottom() {
        let picker = ColorPickerModule::new("picker", rect(0, 0), UiPalette::default());
        let group = picker.draw();

        // order index 0 sits at the highest content row (local y = 5, below
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
