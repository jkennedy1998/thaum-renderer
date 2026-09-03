use crate::{
    Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight,
    GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, Module, ModulePointerButton,
    ModulePointerEvent, ModuleRect, PanelChrome, RawInput, UiColorRole, UiPalette, WorldPoint,
};

/// One listed control: the named action plus presentation metadata. The
/// consumer owns the list; the module only renders, captures, and routes.
#[derive(Debug, Clone)]
pub struct ControlActionRow {
    pub action: crate::ActionName,
    pub label: String,
    pub category: String,
}

/// One visual slot in the panel: a category header or a listed action row.
#[derive(Debug, Clone, PartialEq)]
enum PanelSlot {
    Header(String),
    Row(usize),
}

fn standard_gizmo_bar() -> GizmoBar {
    GizmoBar::new(vec![
        GizmoKind::Move,
        GizmoKind::Close,
        GizmoKind::Resize,
        GizmoKind::Seamless,
    ])
}

/// The big-list controls panel: every declared action with its current
/// binding, click-to-rebind capture, and inline conflict markers. Ships no
/// binding behavior — the consumer wires the getters/setter, so this module
/// works for any program's control map.
///
/// Capture is keyboard-only: keys are the raw inputs that arrive outside the
/// module pointer/wheel routing. Pointer and wheel bindings stay owned by
/// their existing live paths.
pub struct ControlsPanelModule {
    id: String,
    rect: ModuleRect,
    palette: UiPalette,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
    rows: Vec<ControlActionRow>,
    waiting_for: Option<crate::ActionName>,
    get_binding_label: Box<dyn Fn(&crate::ActionName) -> String>,
    get_conflicts: Box<dyn Fn(&crate::ActionName) -> Vec<String>>,
    set_binding: Box<dyn Fn(&crate::ActionName, Option<RawInput>)>,
}

impl ControlsPanelModule {
    pub fn new(
        id: impl Into<String>,
        rect: ModuleRect,
        palette: UiPalette,
        rows: Vec<ControlActionRow>,
        get_binding_label: impl Fn(&crate::ActionName) -> String + 'static,
        get_conflicts: impl Fn(&crate::ActionName) -> Vec<String> + 'static,
        set_binding: impl Fn(&crate::ActionName, Option<RawInput>) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            rect,
            palette,
            gizmos: standard_gizmo_bar(),
            gizmo_state: GizmoState::new(),
            hidden: false,
            rows,
            waiting_for: None,
            get_binding_label: Box::new(get_binding_label),
            get_conflicts: Box::new(get_conflicts),
            set_binding: Box::new(set_binding),
        }
    }

    /// The action currently waiting for a captured key, if any.
    pub fn waiting_action(&self) -> Option<&crate::ActionName> {
        self.waiting_for.as_ref()
    }

    /// Feed one captured key label. Returns `true` when the module was
    /// waiting and consumed the key as a new binding.
    pub fn capture_key(&mut self, label: &str) -> bool {
        let Some(action) = self.waiting_for.take() else {
            return false;
        };
        (self.set_binding)(&action, Some(RawInput::Key(label.to_string())));
        true
    }

    /// Cancel a pending capture without rebinding.
    pub fn cancel_capture(&mut self) {
        self.waiting_for = None;
    }

    /// Visual slot -> what occupies it. Shared by draw and hit-testing so
    /// what you click is what you see.
    fn slot_table(&self) -> Vec<PanelSlot> {
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let capacity = (content_height - 1).max(0) as usize;
        let mut slots: Vec<PanelSlot> = Vec::new();
        let mut last_category = String::new();
        for (index, row) in self.rows.iter().enumerate() {
            if slots.len() >= capacity {
                break;
            }
            if row.category != last_category {
                last_category = row.category.clone();
                slots.push(PanelSlot::Header(row.category.clone()));
                if slots.len() >= capacity {
                    break;
                }
            }
            slots.push(PanelSlot::Row(index));
        }
        slots
    }

    fn row_slot(&self, index: usize) -> Option<(i32, i32)> {
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let (content_x, content_y) = PanelChrome::content_origin();
        let y = content_y + 1 + index as i32;
        if y >= content_y + content_height {
            return None;
        }
        Some((content_x, y))
    }

    /// Rect-local hit-test: which listed row sits at this local position.
    fn row_index_at(&self, local_x: i32, local_y: i32) -> Option<usize> {
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let (content_x, content_y) = PanelChrome::content_origin();
        if local_x < content_x || local_y <= content_y {
            return None;
        }
        let slot = (local_y - content_y - 1) as usize;
        if local_y >= content_y + content_height {
            return None;
        }
        match self.slot_table().get(slot) {
            Some(PanelSlot::Row(index)) => Some(*index),
            _ => None,
        }
    }

    fn write_cells(
        cells: &mut Vec<Cell>,
        x: i32,
        y: i32,
        text: &str,
        color: CellColor,
        max_x: i32,
    ) {
        for (column, glyph) in text.chars().enumerate() {
            let cell_x = x + column as i32;
            if cell_x >= max_x {
                break;
            }
            cells.push(Cell {
                position: CellPoint {
                    x: cell_x,
                    y,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(glyph),
                color,
                weight: CellWeight::from_index_clamped(1),
                ..Cell::default()
            });
        }
    }
}

impl Module for ControlsPanelModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    fn is_hidden(&self) -> bool {
        self.hidden
    }

    fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
        if hidden {
            self.waiting_for = None;
        }
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
                        .with_title("CONTROLS")
                        .with_title_start_x(self.gizmos.title_start_x()),
                    &self.palette,
                )
                .cells()
        };
        if self.gizmo_state.should_draw_gizmo_bar() {
            cells.extend(self.gizmos.cells(self.rect, &self.gizmo_state, &self.palette));
        }

        let (content_width, _) = PanelChrome::content_size(self.rect);
        let (content_x, _) = PanelChrome::content_origin();
        let max_x = content_x + content_width - 1;
        let bright = self.palette.get(UiColorRole::Bright);
        let medium = self.palette.get(UiColorRole::Medium);
        let vivid = self.palette.get(UiColorRole::Vivid);
        let muted = self.palette.get(UiColorRole::Dimmest);

        let mut last_category = String::new();
        for (slot, panel_slot) in self.slot_table().into_iter().enumerate() {
            let Some((_, row_y)) = self.row_slot(slot) else {
                break;
            };
            match panel_slot {
                PanelSlot::Header(category) => {
                    last_category = category.clone();
                    Self::write_cells(&mut cells, content_x + 1, row_y, &category, bright, max_x);
                    continue;
                }
                PanelSlot::Row(index) => {
                    let row = &self.rows[index];
                    let _ = last_category;
                    let binding = (self.get_binding_label)(&row.action);
                    let conflicts = (self.get_conflicts)(&row.action);
                    let waiting = self
                        .waiting_for
                        .as_ref()
                        .is_some_and(|waiting| waiting == &row.action);
                    let binding_text = if waiting {
                        "<PRESS A KEY>".to_string()
                    } else {
                        let marker = if conflicts.is_empty() { "" } else { " !" };
                        format!("{marker}{binding}")
                    };
                    let binding_x = max_x - binding_text.chars().count() as i32;
                    let label_color = if waiting { vivid } else { medium };
                    Self::write_cells(
                        &mut cells,
                        content_x + 1,
                        row_y,
                        &row.label,
                        label_color,
                        binding_x,
                    );
                    let binding_color = if waiting {
                        vivid
                    } else if conflicts.is_empty() {
                        muted
                    } else {
                        vivid
                    };
                    Self::write_cells(&mut cells, binding_x, row_y, &binding_text, binding_color, max_x);
                }
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
                let local_x = x - self.rect.x0;
                let local_y = y - self.rect.y0;
                let Some(index) = self.row_index_at(local_x, local_y) else {
                    return;
                };
                let action = self.rows[index].action.clone();
                match button {
                    ModulePointerButton::Left => {
                        if self.waiting_for.as_ref() == Some(&action) {
                            self.waiting_for = None;
                        } else {
                            self.waiting_for = Some(action);
                        }
                    }
                    ModulePointerButton::Right => {
                        self.waiting_for = None;
                        (self.set_binding)(&action, None);
                    }
                    ModulePointerButton::Middle => {}
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

    fn on_key_capture(&mut self, label: &str) -> bool {
        self.capture_key(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ActionBindingMap;
    use std::{cell::RefCell, rc::Rc};

    fn palette() -> UiPalette {
        UiPalette::default()
    }

    fn rect() -> ModuleRect {
        ModuleRect { x0: 0, y0: 0, x1: 40, y1: 12 }
    }

    fn module(
        bindings: Rc<RefCell<ActionBindingMap>>,
        set_calls: Rc<RefCell<Vec<(String, Option<RawInput>)>>>,
    ) -> ControlsPanelModule {
        let rows = vec![
            ControlActionRow {
                action: crate::ActionName::new("painter_select_pencil"),
                label: "Select Pencil".to_string(),
                category: "tools".to_string(),
            },
            ControlActionRow {
                action: crate::ActionName::new("painter_zoom_in"),
                label: "Zoom In".to_string(),
                category: "camera".to_string(),
            },
        ];
        ControlsPanelModule::new(
            "controls_panel_test",
            rect(),
            palette(),
            rows,
            move |action| {
                bindings
                    .borrow()
                    .bindings_for(action)
                    .first()
                    .map(crate::format_raw_input)
                    .unwrap_or_else(|| "unbound".to_string())
            },
            |_| Vec::new(),
            move |action, binding| {
                set_calls
                    .borrow_mut()
                    .push((action.0.clone(), binding));
            },
        )
    }

    #[test]
    fn capture_key_consumes_the_wait_and_routes_the_binding() {
        let set_calls = Rc::new(RefCell::new(Vec::new()));
        let mut panel = module(
            Rc::new(RefCell::new(ActionBindingMap::new())),
            set_calls.clone(),
        );
        assert!(!panel.capture_key("P"), "no wait, no capture");
        panel.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 3,
            button: ModulePointerButton::Left,
        });
        assert_eq!(
            panel.waiting_action().map(|action| action.0.as_str()),
            Some("painter_select_pencil")
        );
        assert!(panel.capture_key("K"));
        assert!(panel.waiting_action().is_none());
        assert_eq!(
            *set_calls.borrow(),
            vec![(
                "painter_select_pencil".to_string(),
                Some(RawInput::Key("K".to_string()))
            )]
        );
    }

    #[test]
    fn right_click_unbinds_without_capture() {
        let set_calls = Rc::new(RefCell::new(Vec::new()));
        let mut panel = module(
            Rc::new(RefCell::new(ActionBindingMap::new())),
            set_calls.clone(),
        );
        panel.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 3,
            button: ModulePointerButton::Right,
        });
        assert!(panel.waiting_action().is_none());
        assert_eq!(
            *set_calls.borrow(),
            vec![("painter_select_pencil".to_string(), None)]
        );
    }

    #[test]
    fn second_left_click_cancels_the_wait() {
        let set_calls = Rc::new(RefCell::new(Vec::new()));
        let mut panel = module(
            Rc::new(RefCell::new(ActionBindingMap::new())),
            set_calls.clone(),
        );
        panel.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 3,
            button: ModulePointerButton::Left,
        });
        panel.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 3,
            button: ModulePointerButton::Left,
        });
        assert!(panel.waiting_action().is_none());
        assert!(set_calls.borrow().is_empty());
    }

    #[test]
    fn draws_binding_labels_and_capture_prompt() {
        let mut defaults = ActionBindingMap::new();
        defaults.bind(
            crate::ActionName::new("painter_select_pencil"),
            RawInput::Key("P".to_string()),
        );
        let bindings = Rc::new(RefCell::new(defaults));
        let mut panel = module(bindings.clone(), Rc::new(RefCell::new(Vec::new())));
        let text = drawn_text(&panel);
        assert!(text.contains("Select Pencil"), "row label rendered");
        assert!(text.contains("P"), "binding label rendered");
        panel.on_pointer_event(ModulePointerEvent::Click {
            x: 2,
            y: 3,
            button: ModulePointerButton::Left,
        });
        let text = drawn_text(&panel);
        assert!(text.contains("<PRESS A KEY>"), "capture prompt rendered");
    }

    fn drawn_text(panel: &ControlsPanelModule) -> String {
        let cells = panel.draw().cells;
        let max_y = cells.keys().map(|point| point.y).max().unwrap_or(0);
        let mut lines = Vec::new();
        for y in 0..=max_y {
            let line: String = cells
                .iter()
                .filter(|(point, cell)| point.y == y && cell.graphic != CellGraphic::None)
                .map(|(_, cell)| match cell.graphic {
                    CellGraphic::Glyph(glyph) => glyph.to_string(),
                    _ => String::new(),
                })
                .collect();
            lines.push(line);
        }
        lines.join("\n")
    }
}
