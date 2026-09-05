use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    Cell, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight,
    GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, Module, ModulePointerButton,
    ModulePointerEvent, ModuleRect, PanelChrome, PersistedModuleUiState, PropertyRows,
    UiColorRole, UiPalette, WorldPoint,
};
use crate::PerspectiveProfile;

/// One editable perspective knob: a label, wheel fine-step, clamped range,
/// and the preset cycle a click walks through.
struct Knob {
    label: &'static str,
    presets: &'static [f32],
    wheel_step: f32,
    min: f32,
    max: f32,
}

const SCALE_KNOB: Knob = Knob {
    label: "scale",
    presets: &[0.0, 0.5, 0.9, 1.2, 1.5, 2.0],
    wheel_step: 0.05,
    min: 0.0,
    max: 2.0,
};

const POSITION_KNOB: Knob = Knob {
    label: "position",
    presets: &[0.0, 0.5, 0.9, 1.2, 1.5, 2.0],
    wheel_step: 0.05,
    min: 0.0,
    max: 2.0,
};

const FLOOR_KNOB: Knob = Knob {
    label: "floor",
    presets: &[0.28, 0.15, 0.0],
    wheel_step: 0.02,
    min: 0.0,
    max: 0.5,
};

const KNOB_COUNT: usize = 3;

fn knob(index: usize) -> &'static Knob {
    match index {
        0 => &SCALE_KNOB,
        1 => &POSITION_KNOB,
        _ => &FLOOR_KNOB,
    }
}

fn knob_value(profile: &PerspectiveProfile, index: usize) -> f32 {
    match index {
        0 => profile.scale_strength,
        1 => profile.position_strength,
        _ => profile.near_floor_fraction,
    }
}

fn set_knob_value(profile: &mut PerspectiveProfile, index: usize, value: f32) {
    match index {
        0 => profile.scale_strength = value,
        1 => profile.position_strength = value,
        _ => profile.near_floor_fraction = value,
    }
}

/// Renderer-owned camera perspective panel: edits a shared
/// [`PerspectiveProfile`] (scale strength, position strength, near-camera
/// floor) so any app that opts into renderer modules can hand its users
/// direct perspective tuning. The host owns the `Camera` and syncs its
/// `perspective` field from the shared cell.
///
/// Wheel over a knob row fine-tunes it; click cycles through presets; wheel
/// anywhere else falls through so viewport camera behavior keeps working.
pub struct CameraPerspectiveModule {
    id: String,
    rect: ModuleRect,
    profile: Rc<RefCell<PerspectiveProfile>>,
    palette: UiPalette,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
}

impl CameraPerspectiveModule {
    pub fn new(id: impl Into<String>, rect: ModuleRect, profile: Rc<RefCell<PerspectiveProfile>>) -> Self {
        Self {
            id: id.into(),
            rect,
            profile,
            palette: UiPalette::default(),
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            hidden: false,
        }
    }

    /// The profile this module edits, for hosts to bind into their camera.
    pub fn profile(&self) -> Rc<RefCell<PerspectiveProfile>> {
        self.profile.clone()
    }

    fn knob_row_y(&self, index: usize) -> i32 {
        PropertyRows::top_row_y(self.rect) - index as i32
    }

    /// Which knob row (if any) is under this screen-space point. Rows stack
    /// downward from the top content row, one line each, matching the
    /// property-rows convention (draw output is module-local, events are
    /// screen-space like `PropertyRows::hit_test`).
    fn knob_row_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.rect.contains(x, y) {
            return None;
        }
        let local_y = y - self.rect.y0;
        (0..KNOB_COUNT).find(|&index| self.knob_row_y(index) == local_y)
    }

    fn apply_wheel(&self, profile: &mut PerspectiveProfile, index: usize, delta_y: f32) {
        let knob = knob(index);
        let step = if delta_y > 0.0 {
            knob.wheel_step
        } else if delta_y < 0.0 {
            -knob.wheel_step
        } else {
            return;
        };
        let value = (knob_value(profile, index) + step).clamp(knob.min, knob.max);
        set_knob_value(profile, index, (value * 100.0).round() / 100.0);
    }

    fn cycle_preset(&self, profile: &mut PerspectiveProfile, index: usize) {
        let knob = knob(index);
        let current = knob_value(profile, index);
        let position = knob
            .presets
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1 - current).abs()
                    .total_cmp(&(b.1 - current).abs())
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        let next = knob.presets[(position + 1) % knob.presets.len()];
        set_knob_value(profile, index, next);
    }
}

impl Module for CameraPerspectiveModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    fn draw(&self) -> CellGroup {
        let mut cells: Vec<Cell> = if self.gizmo_state.is_seamless() {
            Vec::new()
        } else {
            self.gizmo_state
                .decorate_panel_chrome(
                    PanelChrome::new(self.rect, &self.palette)
                        .with_title("PERSPECTIVE")
                        .with_title_start_x(self.gizmos.title_start_x()),
                    &self.palette,
                )
                .cells()
        };
        if self.gizmo_state.should_draw_gizmo_bar() {
            cells.extend(self.gizmos.cells(self.rect, &self.gizmo_state, &self.palette));
        }

        let (content_x, content_y) = PanelChrome::content_origin();
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let top_row_y = content_y + (content_height - 1).max(0);
        let value_x = content_x + 9;
        let profile = self.profile.borrow();

        for index in 0..KNOB_COUNT {
            let y = top_row_y - index as i32;
            for (offset, glyph) in knob(index).label.chars().enumerate() {
                cells.push(Cell {
                    position: CellPoint {
                        x: content_x + offset as i32,
                        y,
                        z: 0,
                    },
                    graphic: CellGraphic::Glyph(glyph),
                    color: self.palette.get(UiColorRole::Medium),
                    weight: CellWeight::from_index_clamped(1),
                    ..Cell::default()
                });
            }
            let value = format!("{:.2}", knob_value(&profile, index));
            for (offset, glyph) in value.chars().enumerate() {
                cells.push(Cell {
                    position: CellPoint {
                        x: value_x + offset as i32,
                        y,
                        z: 0,
                    },
                    graphic: CellGraphic::Glyph(glyph),
                    color: self.palette.get(UiColorRole::Vivid),
                    weight: CellWeight::from_index_clamped(2),
                    ..Cell::default()
                });
            }
        }

        // Hint row on the bottom content line, dimmed.
        let hint_y = content_y;
        for (offset, glyph) in "wheel fine  click preset".chars().enumerate() {
            cells.push(Cell {
                position: CellPoint {
                    x: content_x + offset as i32,
                    y: hint_y,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(glyph),
                color: self.palette.get(UiColorRole::Dimmest),
                weight: CellWeight::from_index_clamped(1),
                ..Cell::default()
            });
        }

        // Group origin is the rect origin (shared module convention): cells
        // are rect-local, so an origin of (0,0) would draw the panel at the
        // screen's bottom-left corner while hit-testing still listens at
        // `self.rect` — making the panel visibly dead to input.
        let mut group = CellGroup::new(WorldPoint {
                x: self.rect.x0,
                y: self.rect.y0,
                z: 0,
            })
            .with_intake_behavior(CellGroupIntakeBehavior::Flat2d);
        group.extend(cells);
        group
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
                if button == ModulePointerButton::Left {
                    if let Some(index) = self.knob_row_at(x, y) {
                        let mut profile = self.profile.borrow_mut();
                        self.cycle_preset(&mut profile, index);
                    }
                }
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

    fn on_wheel(&mut self, x: i32, y: i32, _delta_x: f32, delta_y: f32) -> bool {
        let Some(index) = self.knob_row_at(x, y) else {
            // Off the knob rows: fall through to viewport camera behavior.
            return false;
        };
        let mut profile = self.profile.borrow_mut();
        self.apply_wheel(&mut profile, index, delta_y);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> ModuleRect {
        ModuleRect {
            x0: 10,
            y0: 10,
            x1: 33,
            y1: 18,
        }
    }

    fn module() -> (CameraPerspectiveModule, Rc<RefCell<PerspectiveProfile>>) {
        make_module()
    }

    fn make_module() -> (CameraPerspectiveModule, Rc<RefCell<PerspectiveProfile>>) {
        let profile = Rc::new(RefCell::new(PerspectiveProfile::default()));
        let module = CameraPerspectiveModule::new("camera_perspective", rect(), profile.clone());
        (module, profile)
    }

    fn scale_row_y() -> i32 {
        // Screen-space: knob rows are module-local inside draw, events are
        // screen-space, so add the rect origin.
        rect().y0 + PropertyRows::top_row_y(rect())
    }

    fn gizmo_row_y() -> i32 {
        // Gizmo glyphs sit on the chrome's top border row: local y = height-1.
        rect().y0 + (rect().y1 - rect().y0) - 1
    }

    #[test]
    fn clicking_the_close_gizmo_hides_the_panel_and_recall_revives_it() {
        let (mut module, _profile) = module();
        assert!(!module.is_hidden());

        module.on_pointer_event(ModulePointerEvent::Click {
            x: rect().x0 + 3,
            y: gizmo_row_y(),
            button: ModulePointerButton::Left,
        });
        assert!(module.is_hidden(), "close gizmo must hide the panel");

        module.set_hidden(false);
        assert!(!module.is_hidden(), "bottom-bar recall must revive it");
    }

    #[test]
    fn move_gizmo_drag_relocates_the_panel_through_pointer_capture() {
        let (mut module, _profile) = module();
        let before = module.rect();

        module.on_pointer_event(ModulePointerEvent::Click {
            x: rect().x0 + 1,
            y: gizmo_row_y(),
            button: ModulePointerButton::Left,
        });
        assert!(module.wants_pointer_capture(), "move drag must hold capture");

        module.on_pointer_event(ModulePointerEvent::Move {
            x: rect().x0 + 4,
            y: gizmo_row_y() + 2,
        });
        assert_eq!(module.rect(), ModuleRect {
            x0: before.x0 + 3,
            y0: before.y0 + 2,
            x1: before.x1 + 3,
            y1: before.y1 + 2,
        });

        module.on_pointer_event(ModulePointerEvent::Up { x: 0, y: 0 });
        assert!(!module.wants_pointer_capture(), "release must end the drag");
    }

    #[test]
    fn panel_ui_state_persists_rect_seamless_and_hidden() {
        let (mut module, _profile) = module();
        module.on_pointer_event(ModulePointerEvent::Click {
            x: rect().x0 + 7,
            y: gizmo_row_y(),
            button: ModulePointerButton::Left,
        });
        assert!(module.gizmo_state.is_seamless());

        let saved = module.persisted_ui_state().expect("ui state snapshot");
        let (mut revived, _profile) = make_module();
        revived.apply_persisted_ui_state(&saved);
        assert_eq!(revived.rect(), module.rect());
        assert!(revived.gizmo_state.is_seamless());
    }

    #[test]
    fn draw_output_is_anchored_at_the_module_rect_not_the_screen_origin() {
        // Regression: a group origin of (0,0) drew the panel at the screen's
        // bottom-left corner while hit-testing stayed at `self.rect`, so the
        // visible panel was dead to input.
        let (module, _profile) = module();
        let group = module.draw();
        let origin = group.origin;
        assert_eq!((origin.x, origin.y), (rect().x0, rect().y0));
    }

    #[test]
    fn wheel_over_a_knob_row_fine_tunes_that_knob() {
        let (mut module, profile) = module();
        let y = scale_row_y();

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!((profile.borrow().scale_strength - 0.95).abs() < 1e-4);
        assert!(module.on_wheel(12, y, 0.0, -1.0));
        assert!((profile.borrow().scale_strength - 0.90).abs() < 1e-4);

        // The other knobs are untouched by scale-row wheels.
        assert!((profile.borrow().position_strength - 0.9).abs() < 1e-4);
        assert!((profile.borrow().near_floor_fraction - 0.28).abs() < 1e-4);
    }

    #[test]
    fn wheel_nudging_clamps_at_the_knob_range() {
        let (mut module, profile) = module();
        let y = scale_row_y();
        for _ in 0..80 {
            module.on_wheel(12, y, 0.0, 1.0);
        }
        assert!((profile.borrow().scale_strength - 2.0).abs() < 1e-4);
        for _ in 0..80 {
            module.on_wheel(12, y, 0.0, -1.0);
        }
        assert!((profile.borrow().scale_strength - 0.0).abs() < 1e-4);
    }

    #[test]
    fn clicking_a_knob_row_cycles_presets_and_wraps() {
        let (mut module, profile) = module();
        let y = scale_row_y();

        // Default 0.9 is a preset: the next click lands on 1.2.
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert!((profile.borrow().scale_strength - 1.2).abs() < 1e-4);
        for _ in 0..5 {
            module.on_pointer_event(ModulePointerEvent::Click {
                x: 12,
                y,
                button: ModulePointerButton::Left,
            });
        }
        // 1.5 -> 2.0 -> 0.0 -> 0.5 -> 0.9: wrapped back around.
        assert!((profile.borrow().scale_strength - 0.9).abs() < 1e-4);
    }

    #[test]
    fn wheel_off_the_knob_rows_falls_through_to_the_viewport() {
        let (mut module, _profile) = module();
        assert!(!module.on_wheel(12, rect().y0 + 1, 0.0, 1.0));
        assert!(!module.on_wheel(12, rect().y1 - 1, 0.0, 1.0));
    }

    #[test]
    fn edits_land_in_the_shared_profile_the_host_binds() {
        let (mut module, profile) = module();
        let floor_y = scale_row_y() - 2;
        module.on_wheel(12, floor_y, 0.0, -1.0);
        assert!((profile.borrow().near_floor_fraction - 0.26).abs() < 1e-4);
        assert_eq!(module.profile(), profile);
    }

    #[test]
    fn draw_renders_knob_labels_and_live_values() {
        let (module, profile) = module();
        profile.borrow_mut().scale_strength = 1.25;
        let group = module.draw();
        // Draw output is module-local: compare against the local top row.
        let label_row_y = PropertyRows::top_row_y(rect());
        let glyphs: Vec<(CellPoint, char)> = group
            .iter_cells()
            .filter_map(|cell| match &cell.graphic {
                CellGraphic::Glyph(glyph) => Some((cell.position, *glyph)),
                _ => None,
            })
            .collect();

        for (row_offset, label) in ["scale", "position", "floor"].iter().enumerate() {
            assert!(glyphs.iter().any(|(position, glyph)| {
                position.y == label_row_y - row_offset as i32
                    && label.starts_with(*glyph)
            }),
            "missing {label}");
        }
        // The live value renders, including the edited one.
        for expected in ['1', '.', '2', '5'] {
            assert!(glyphs.iter().any(|(position, glyph)| {
                position.y == label_row_y && glyph == &expected
            }));
        }
    }
}
