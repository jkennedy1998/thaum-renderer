use crate::{
    Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight,
    GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, Hotspot, Module, ModulePointerButton,
    ModulePointerEvent, ModuleRect, PanelChrome, PersistedModuleUiState, UiColorRole, UiPalette,
    WorldPoint,
};

fn rgb_cell(rgb: [u8; 3]) -> CellColor {
    CellColor::Flat([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    ])
}

pub struct UiCustomizationModule {
    id: String,
    rect: ModuleRect,
    palette: UiPalette,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
    selected_role: UiColorRole,
    get_left_rgb: Box<dyn Fn() -> [u8; 3]>,
    get_right_rgb: Box<dyn Fn() -> [u8; 3]>,
}

impl UiCustomizationModule {
    pub fn new(
        id: impl Into<String>,
        rect: ModuleRect,
        palette: UiPalette,
        get_left_rgb: impl Fn() -> [u8; 3] + 'static,
        get_right_rgb: impl Fn() -> [u8; 3] + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            rect,
            palette,
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            hidden: false,
            selected_role: UiColorRole::Vivid,
            get_left_rgb: Box::new(get_left_rgb),
            get_right_rgb: Box::new(get_right_rgb),
        }
    }

    fn row_y(&self, index: usize) -> i32 {
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let (_, content_y) = PanelChrome::content_origin();
        content_y + content_height - 2 - index as i32
    }

    fn role_at(&self, x: i32, y: i32) -> Option<UiColorRole> {
        let local_x = x - self.rect.x0;
        let local_y = y - self.rect.y0;
        let (content_x, _) = PanelChrome::content_origin();
        let (content_width, _) = PanelChrome::content_size(self.rect);
        if local_x < content_x || local_x >= content_x + content_width || local_y <= 0 {
            return None;
        }
        UiColorRole::ALL
            .iter()
            .enumerate()
            .find(|(index, _)| local_y == self.row_y(*index))
            .map(|(_, role)| *role)
    }

    fn button_rgb(&self, button: ModulePointerButton) -> Option<[u8; 3]> {
        match button {
            ModulePointerButton::Left => Some((self.get_left_rgb)()),
            ModulePointerButton::Right => Some((self.get_right_rgb)()),
            ModulePointerButton::Middle => None,
        }
    }
}

impl Module for UiCustomizationModule {
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
        let mut cells: Vec<Cell> = if self.gizmo_state.is_seamless() {
            Vec::new()
        } else {
            self.gizmo_state
                .decorate_panel_chrome(
                    PanelChrome::new(self.rect, &self.palette)
                        .with_title("UI COLORS")
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

        let bright = self.palette.get(UiColorRole::Bright);
        let medium = self.palette.get(UiColorRole::Medium);
        let vivid = self.palette.get(UiColorRole::Vivid);
        let left_rgb = (self.get_left_rgb)();
        let right_rgb = (self.get_right_rgb)();
        let helper = "L/R CLICK = APPLY";
        for (index, glyph) in helper.chars().enumerate() {
            cells.push(Cell {
                position: CellPoint {
                    x: 1 + index as i32,
                    y: 1,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(glyph),
                color: bright,
                weight: CellWeight::from_index_clamped(1),
                ..Cell::default()
            });
        }
        for (x, label, rgb) in [(1, 'L', left_rgb), (5, 'R', right_rgb)] {
            cells.push(Cell {
                position: CellPoint { x, y: 0, z: 0 },
                graphic: CellGraphic::Glyph(label),
                color: bright,
                weight: CellWeight::from_index_clamped(2),
                ..Cell::default()
            });
            cells.push(Cell {
                position: CellPoint {
                    x: x + 2,
                    y: 0,
                    z: 0,
                },
                graphic: CellGraphic::Glyph('█'),
                color: rgb_cell(rgb),
                weight: CellWeight::from_index_clamped(3),
                ..Cell::default()
            });
        }

        for (index, role) in UiColorRole::ALL.iter().enumerate() {
            let y = self.row_y(index);
            let is_selected = *role == self.selected_role;
            cells.push(Cell {
                position: CellPoint { x: 1, y, z: 0 },
                graphic: CellGraphic::Glyph(if is_selected { '▶' } else { ' ' }),
                color: if is_selected { vivid } else { medium },
                weight: CellWeight::from_index_clamped(if is_selected { 2 } else { 1 }),
                ..Cell::default()
            });
            cells.push(Cell {
                position: CellPoint { x: 3, y, z: 0 },
                graphic: CellGraphic::Glyph('█'),
                color: self.palette.get(*role),
                weight: CellWeight::from_index_clamped(3),
                ..Cell::default()
            });
            for (column, glyph) in role.label().chars().enumerate() {
                let x = 5 + column as i32;
                if x > self.rect.x1 - self.rect.x0 - 1 {
                    break;
                }
                cells.push(Cell {
                    position: CellPoint { x, y, z: 0 },
                    graphic: CellGraphic::Glyph(glyph),
                    color: if is_selected { bright } else { medium },
                    weight: CellWeight::from_index_clamped(if is_selected { 2 } else { 1 }),
                    ..Cell::default()
                });
            }
        }

        CellGroup::from_cells(origin, cells).with_intake_behavior(CellGroupIntakeBehavior::Flat2d)
    }

    fn on_pointer_event(&mut self, event: ModulePointerEvent) {
        match event {
            ModulePointerEvent::Click { x, y, button } => {
                if let Some(outcome) = self.gizmo_state.handle_click(&self.gizmos, self.rect, x, y)
                {
                    if outcome == GizmoClickOutcome::Gizmo(GizmoKind::Close) {
                        self.hidden = true;
                    }
                    return;
                }
                let Some(role) = self.role_at(x, y) else {
                    return;
                };
                self.selected_role = role;
                let Some(rgb) = self.button_rgb(button) else {
                    return;
                };
                self.palette.set_rgb(role, rgb);
            }
            ModulePointerEvent::Move { x, y } => {
                self.gizmo_state.note_pointer(&self.gizmos, self.rect, x, y);
                if let Some(next_rect) = self.gizmo_state.drag_rect(x, y) {
                    self.rect = next_rect;
                }
            }
            ModulePointerEvent::Up { .. } => self.gizmo_state.end_drag(),
            ModulePointerEvent::Enter => self.gizmo_state.set_hovered(true),
            ModulePointerEvent::Leave => self.gizmo_state.set_hovered(false),
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

    fn rect() -> ModuleRect {
        ModuleRect {
            x0: 0,
            y0: 0,
            x1: 20,
            y1: 10,
        }
    }

    #[test]
    fn left_click_applies_the_left_color_to_the_clicked_role() {
        let palette = UiPalette::default();
        let mut module =
            UiCustomizationModule::new("ui", rect(), palette.clone(), || [1, 2, 3], || [4, 5, 6]);

        let background_y = module.rect.y0 + module.row_y(0);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 3,
            y: background_y,
            button: ModulePointerButton::Left,
        });

        assert_eq!(palette.get_rgb(UiColorRole::Background), [1, 2, 3]);
    }

    #[test]
    fn right_click_applies_the_right_color_to_the_clicked_role() {
        let palette = UiPalette::default();
        let mut module =
            UiCustomizationModule::new("ui", rect(), palette.clone(), || [1, 2, 3], || [7, 8, 9]);

        let dimmest_y = module.rect.y0 + module.row_y(1);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 3,
            y: dimmest_y,
            button: ModulePointerButton::Right,
        });

        assert_eq!(palette.get_rgb(UiColorRole::Dimmest), [7, 8, 9]);
    }

    #[test]
    fn clicking_helper_rows_does_not_change_the_palette() {
        let palette = UiPalette::default();
        let before = palette.get_rgb(UiColorRole::Vivid);
        let mut module =
            UiCustomizationModule::new("ui", rect(), palette.clone(), || [1, 2, 3], || [7, 8, 9]);

        module.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 1,
            button: ModulePointerButton::Left,
        });

        assert_eq!(palette.get_rgb(UiColorRole::Vivid), before);
    }
}
