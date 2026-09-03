use crate::{CellGroup, CellGroupIntakeBehavior, PersistedModuleUiState, WorldPoint};

/// A screen-space rect in cell units, inclusive on both ends, matching the old
/// mono_ui `Rect` convention (bottom-left coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl ModuleRect {
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
}

/// Which mouse button triggered a module click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulePointerButton {
    Left,
    Right,
    Middle,
}

/// v1 pointer lifecycle a module may react to. `Move` is continuous pointer
/// position delivered only while a module holds pointer capture (see
/// `ModuleRegistry::dispatch_captured_pointer_move`) — used for drag
/// sessions like gizmo move/resize. No keyboard or focus yet — those land in
/// `shared/` once a real module needs them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulePointerEvent {
    Enter,
    Leave,
    Down {
        x: i32,
        y: i32,
    },
    Up {
        x: i32,
        y: i32,
    },
    Click {
        x: i32,
        y: i32,
        button: ModulePointerButton,
    },
    Move {
        x: i32,
        y: i32,
    },
}

/// One renderer-owned interactive UI panel. `draw` is the only required
/// behavior; a module that draws nothing interactive is still a valid module.
pub trait Module {
    fn id(&self) -> &str;
    fn rect(&self) -> ModuleRect;

    /// Whether this module is currently hidden from draw and hit-test while
    /// keeping its user-session state alive for later re-show.
    fn is_hidden(&self) -> bool {
        false
    }

    /// Toggle whether this module is hidden from draw and hit-test.
    fn set_hidden(&mut self, _hidden: bool) {}

    /// Render this module's content as one `CellGroup`, positioned at the
    /// module's rect origin and always flat against the screen (`Flat2d`).
    fn draw(&self) -> CellGroup;

    /// React to a pointer lifecycle event already known to be within this
    /// module's rect. No-op by default.
    fn on_pointer_event(&mut self, _event: ModulePointerEvent) {}

    /// React to one wheel event already known to be within this module's
    /// rect. Returns `true` when the module consumed it and outer routing
    /// should stop; `false` by default so callers may fall back to broader
    /// viewport/camera scroll behavior.
    fn on_wheel(&mut self, _x: i32, _y: i32, _delta_x: f32, _delta_y: f32) -> bool {
        false
    }

    /// Whether this module wants to keep receiving pointer events (`Move`,
    /// then `Up`) regardless of where the pointer is, until it says
    /// otherwise — set true for the duration of a gizmo drag session.
    /// `false` by default.
    fn wants_pointer_capture(&self) -> bool {
        false
    }

    /// Whether this module has asked to be removed (its close gizmo was
    /// clicked). `ModuleRegistry::remove_closed_modules` purges modules
    /// where this is true. `false` by default.
    fn is_closed(&self) -> bool {
        false
    }

    /// Offer one captured key label to the module (input-capture flows such
    /// as the controls panel's rebind wait). Returns `true` when the module
    /// consumed the key. `false` by default.
    fn on_key_capture(&mut self, _label: &str) -> bool {
        false
    }

    /// Snapshot the module's user-facing UI/session state when this module
    /// wants renderer-owned persistence support.
    fn persisted_ui_state(&self) -> Option<PersistedModuleUiState> {
        None
    }

    /// Apply one previously saved user-facing UI/session state blob back to
    /// the module.
    fn apply_persisted_ui_state(&mut self, _state: &PersistedModuleUiState) {}
}

/// A minimal concrete module used to prove the shape: an empty panel that
/// draws only its rect origin as one visible cell-group with no content.
pub struct BlankPanelModule {
    id: String,
    rect: ModuleRect,
}

impl BlankPanelModule {
    pub fn new(id: impl Into<String>, rect: ModuleRect) -> Self {
        Self {
            id: id.into(),
            rect,
        }
    }
}

impl Module for BlankPanelModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    fn draw(&self) -> CellGroup {
        CellGroup::new(WorldPoint {
            x: self.rect.x0,
            y: self.rect.y0,
            z: 0,
        })
        .with_intake_behavior(CellGroupIntakeBehavior::Flat2d)
    }
}

/// Registers modules by id, tracks z-order (draw/hit-test order), and
/// resolves which module (if any) sits at one screen point, topmost first.
#[derive(Default)]
pub struct ModuleRegistry {
    order: Vec<Box<dyn Module>>,
    captured: Option<String>,
    hovered: Option<String>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, module: Box<dyn Module>) {
        self.unregister(module.id());
        self.order.push(module);
    }

    pub fn unregister(&mut self, id: &str) -> bool {
        let before = self.order.len();
        self.order.retain(|module| module.id() != id);
        self.order.len() != before
    }

    pub fn bring_to_front(&mut self, id: &str) -> bool {
        if let Some(index) = self.order.iter().position(|module| module.id() == id) {
            let module = self.order.remove(index);
            self.order.push(module);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Topmost module (last in z-order) containing the given screen point, if any.
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&dyn Module> {
        self.order
            .iter()
            .rev()
            .find(|module| !module.is_hidden() && module.rect().contains(x, y))
            .map(|module| module.as_ref())
    }

    /// All registered modules in draw (back-to-front) order, for a consumer
    /// building a per-frame composition from the registry.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Module> {
        self.order
            .iter()
            .filter(|module| !module.is_hidden())
            .map(|module| module.as_ref())
    }

    pub fn persisted_ui_state(&self) -> Vec<PersistedModuleUiState> {
        self.order
            .iter()
            .filter_map(|module| module.persisted_ui_state())
            .collect()
    }

    pub fn apply_persisted_ui_state(&mut self, states: &[PersistedModuleUiState]) {
        for state in states {
            if let Some(module) = self
                .order
                .iter_mut()
                .find(|module| module.id() == state.module_id)
            {
                module.apply_persisted_ui_state(state);
            }
        }
    }

    pub fn is_hidden(&self, id: &str) -> Option<bool> {
        self.order
            .iter()
            .find(|module| module.id() == id)
            .map(|module| module.is_hidden())
    }

    pub fn set_hidden(&mut self, id: &str, hidden: bool) -> bool {
        let Some(module) = self.order.iter_mut().find(|module| module.id() == id) else {
            return false;
        };
        module.set_hidden(hidden);
        if hidden {
            if self.captured.as_deref() == Some(id) {
                self.captured = None;
            }
            if self.hovered.as_deref() == Some(id) {
                self.hovered = None;
            }
        }
        true
    }

    /// Deliver a pointer event to the topmost module containing `(x, y)`,
    /// if any. Returns the id of the module that handled it. If that module
    /// reports `wants_pointer_capture() == true` afterward, it becomes the
    /// captured module for subsequent `dispatch_captured_pointer_move`/`_up`
    /// calls, regardless of where the pointer moves.
    pub fn dispatch_pointer_event_at(
        &mut self,
        x: i32,
        y: i32,
        event: ModulePointerEvent,
    ) -> Option<&str> {
        let index = self
            .order
            .iter()
            .rposition(|module| !module.is_hidden() && module.rect().contains(x, y))?;
        self.order[index].on_pointer_event(event);
        if self.order[index].wants_pointer_capture() {
            self.captured = Some(self.order[index].id().to_string());
        }
        Some(self.order[index].id())
    }

    /// Deliver one wheel event to the topmost module containing `(x, y)`, if
    /// any. Returns the id only when that module consumed the wheel.
    pub fn dispatch_wheel_at(
        &mut self,
        x: i32,
        y: i32,
        delta_x: f32,
        delta_y: f32,
    ) -> Option<&str> {
        let index = self
            .order
            .iter()
            .rposition(|module| !module.is_hidden() && module.rect().contains(x, y))?;
        self.order[index]
            .on_wheel(x, y, delta_x, delta_y)
            .then_some(self.order[index].id())
    }

    /// Deliver continuous pointer movement to the captured module (see
    /// `dispatch_pointer_event_at`), bypassing hit-testing so a drag session
    /// keeps working even once the pointer leaves the module's rect. A no-op
    /// if no module currently holds capture.
    pub fn dispatch_captured_pointer_move(&mut self, x: i32, y: i32) -> Option<&str> {
        let id = self.captured.clone()?;
        let index = self.order.iter().position(|module| module.id() == id)?;
        self.order[index].on_pointer_event(ModulePointerEvent::Move { x, y });
        Some(self.order[index].id())
    }

    /// Deliver a pointer-up to the captured module and release capture.
    /// A no-op if no module currently holds capture.
    pub fn dispatch_captured_pointer_up(&mut self, x: i32, y: i32) -> Option<&str> {
        let id = self.captured.take()?;
        let index = self.order.iter().position(|module| module.id() == id)?;
        self.order[index].on_pointer_event(ModulePointerEvent::Up { x, y });
        Some(self.order[index].id())
    }

    /// Whether any module currently holds pointer capture for an active drag
    /// interaction such as move/resize.
    /// Offer one captured key label to registered modules in registration
    /// order. Returns the consuming module's id, if any.
    pub fn dispatch_key_capture(&mut self, label: &str) -> Option<&str> {
        for module in self.order.iter_mut() {
            if module.on_key_capture(label) {
                return Some(module.id());
            }
        }
        None
    }

    pub fn is_pointer_captured(&self) -> bool {
        self.captured.is_some()
    }

    /// Remove every module that has asked to be closed (`Module::is_closed`
    /// is true), releasing capture/hover first if that module held either.
    /// Hidden modules are intentionally retained.
    pub fn remove_closed_modules(&mut self) {
        let is_closed = |id: &str| {
            self.order
                .iter()
                .any(|module| module.id() == id && module.is_closed())
        };
        if self.captured.as_deref().is_some_and(is_closed) {
            self.captured = None;
        }
        if self.hovered.as_deref().is_some_and(is_closed) {
            self.hovered = None;
        }
        self.order.retain(|module| !module.is_closed());
    }

    /// Update which module (if any) the pointer currently hovers, given its
    /// current absolute position — `None` if the pointer is outside the
    /// window or otherwise unknown this frame. Delivers `Leave` to whichever
    /// module was previously hovered (if it changed) and `Enter` to the
    /// newly hovered one (topmost under the point, same as `hit_test`), so a
    /// seamless module can reveal its gizmo bar only while moused over.
    /// Also delivers `Move { x, y }` to the currently hovered module so it
    /// can refine hover affordances inside its own rect even when it does not
    /// hold pointer capture. Returns the id of the now-hovered module, if any.
    pub fn update_hover_at(&mut self, point: Option<(i32, i32)>) -> Option<&str> {
        let hit_id =
            point.and_then(|(x, y)| self.hit_test(x, y).map(|module| module.id().to_string()));

        if hit_id != self.hovered {
            if let Some(previous_id) = self.hovered.take() {
                if let Some(index) = self
                    .order
                    .iter()
                    .position(|module| module.id() == previous_id)
                {
                    self.order[index].on_pointer_event(ModulePointerEvent::Leave);
                }
            }
            if let Some(next_id) = &hit_id {
                if let Some(index) = self.order.iter().position(|module| module.id() == *next_id) {
                    self.order[index].on_pointer_event(ModulePointerEvent::Enter);
                }
            }
            self.hovered = hit_id;
        }

        if let (Some(id), Some((x, y))) = (self.hovered.as_deref(), point) {
            if let Some(index) = self.order.iter().position(|module| module.id() == id) {
                self.order[index].on_pointer_event(ModulePointerEvent::Move { x, y });
            }
        }

        self.hovered.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> ModuleRect {
        ModuleRect { x0, y0, x1, y1 }
    }

    #[test]
    fn blank_panel_draws_a_flat_2d_cell_group_at_its_origin() {
        let panel = BlankPanelModule::new("panel_a", rect(2, 3, 10, 8));
        let group = panel.draw();

        assert_eq!(group.origin, WorldPoint { x: 2, y: 3, z: 0 });
        assert_eq!(group.intake_behavior, CellGroupIntakeBehavior::Flat2d);
    }

    #[test]
    fn registry_hit_test_finds_topmost_module_first() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(BlankPanelModule::new("back", rect(0, 0, 10, 10))));
        registry.register(Box::new(BlankPanelModule::new("front", rect(2, 2, 6, 6))));

        assert_eq!(registry.hit_test(3, 3).map(Module::id), Some("front"));
        assert_eq!(registry.hit_test(8, 8).map(Module::id), Some("back"));
        assert_eq!(registry.hit_test(20, 20).map(Module::id), None);
    }

    #[test]
    fn bring_to_front_changes_hit_test_precedence() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(BlankPanelModule::new("a", rect(0, 0, 10, 10))));
        registry.register(Box::new(BlankPanelModule::new("b", rect(0, 0, 10, 10))));

        assert_eq!(registry.hit_test(1, 1).map(Module::id), Some("b"));
        registry.bring_to_front("a");
        assert_eq!(registry.hit_test(1, 1).map(Module::id), Some("a"));
    }

    #[test]
    fn unregister_removes_module_from_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(BlankPanelModule::new("a", rect(0, 0, 10, 10))));
        assert_eq!(registry.len(), 1);

        assert!(registry.unregister("a"));
        assert!(registry.is_empty());
        assert!(!registry.unregister("a"));
    }

    #[test]
    fn iter_yields_modules_in_registration_order() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(BlankPanelModule::new("a", rect(0, 0, 1, 1))));
        registry.register(Box::new(BlankPanelModule::new("b", rect(0, 0, 1, 1))));

        let ids: Vec<&str> = registry.iter().map(Module::id).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    struct RecordingModule {
        id: String,
        rect: ModuleRect,
        last_event: Option<ModulePointerEvent>,
        last_wheel: Option<(i32, i32, f32, f32)>,
        capturing: bool,
        closed: bool,
        hidden: bool,
    }

    impl RecordingModule {
        fn new(id: &str, rect: ModuleRect) -> Self {
            Self {
                id: id.to_string(),
                rect,
                last_event: None,
                last_wheel: None,
                capturing: false,
                closed: false,
                hidden: false,
            }
        }
    }

    impl Module for RecordingModule {
        fn id(&self) -> &str {
            &self.id
        }

        fn rect(&self) -> ModuleRect {
            self.rect
        }

        fn draw(&self) -> CellGroup {
            CellGroup::new(WorldPoint::origin())
        }

        fn is_hidden(&self) -> bool {
            self.hidden
        }

        fn set_hidden(&mut self, hidden: bool) {
            self.hidden = hidden;
        }

        fn on_pointer_event(&mut self, event: ModulePointerEvent) {
            self.last_event = Some(event);
        }

        fn on_wheel(&mut self, x: i32, y: i32, delta_x: f32, delta_y: f32) -> bool {
            self.last_wheel = Some((x, y, delta_x, delta_y));
            true
        }

        fn wants_pointer_capture(&self) -> bool {
            self.capturing
        }

        fn is_closed(&self) -> bool {
            self.closed
        }
    }

    #[test]
    fn dispatch_pointer_event_at_delivers_to_topmost_module_under_the_point() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("back", rect(0, 0, 10, 10))));
        registry.register(Box::new(RecordingModule::new("front", rect(2, 2, 6, 6))));

        let handled = registry.dispatch_pointer_event_at(
            3,
            3,
            ModulePointerEvent::Click {
                x: 3,
                y: 3,
                button: ModulePointerButton::Left,
            },
        );
        assert_eq!(handled, Some("front"));
    }

    #[test]
    fn dispatch_pointer_event_at_returns_none_when_no_module_contains_the_point() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(BlankPanelModule::new("a", rect(0, 0, 1, 1))));

        let handled = registry.dispatch_pointer_event_at(
            9,
            9,
            ModulePointerEvent::Click {
                x: 9,
                y: 9,
                button: ModulePointerButton::Left,
            },
        );
        assert_eq!(handled, None);
    }

    #[test]
    fn dispatch_wheel_at_delivers_to_the_topmost_module_under_the_point() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("back", rect(0, 0, 10, 10))));
        registry.register(Box::new(RecordingModule::new("front", rect(2, 2, 6, 6))));

        let handled = registry.dispatch_wheel_at(3, 3, -2.0, 1.0);

        assert_eq!(handled, Some("front"));
    }

    #[test]
    fn a_module_that_wants_capture_keeps_receiving_move_events_outside_its_rect() {
        let mut registry = ModuleRegistry::new();
        let mut captor = RecordingModule::new("captor", rect(0, 0, 4, 4));
        captor.capturing = true;
        registry.register(Box::new(captor));

        registry.dispatch_pointer_event_at(
            1,
            1,
            ModulePointerEvent::Click {
                x: 1,
                y: 1,
                button: ModulePointerButton::Left,
            },
        );
        let handled = registry.dispatch_captured_pointer_move(999, 999);

        assert_eq!(handled, Some("captor"));
    }

    #[test]
    fn dispatch_captured_pointer_move_is_a_noop_when_nothing_is_captured() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("a", rect(0, 0, 4, 4))));

        assert_eq!(registry.dispatch_captured_pointer_move(1, 1), None);
    }

    #[test]
    fn dispatch_captured_pointer_up_delivers_up_and_releases_capture() {
        let mut registry = ModuleRegistry::new();
        let mut captor = RecordingModule::new("captor", rect(0, 0, 4, 4));
        captor.capturing = true;
        registry.register(Box::new(captor));

        registry.dispatch_pointer_event_at(
            1,
            1,
            ModulePointerEvent::Click {
                x: 1,
                y: 1,
                button: ModulePointerButton::Left,
            },
        );
        assert!(registry.is_pointer_captured());
        let handled = registry.dispatch_captured_pointer_up(2, 2);
        assert_eq!(handled, Some("captor"));

        // Capture was released: a further move goes nowhere.
        assert_eq!(registry.dispatch_captured_pointer_move(3, 3), None);
        assert!(!registry.is_pointer_captured());
    }

    #[test]
    fn remove_closed_modules_purges_only_closed_modules() {
        let mut registry = ModuleRegistry::new();
        let mut closing = RecordingModule::new("closing", rect(0, 0, 4, 4));
        closing.closed = true;
        registry.register(Box::new(closing));
        registry.register(Box::new(RecordingModule::new("staying", rect(0, 0, 4, 4))));

        registry.remove_closed_modules();

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.hit_test(1, 1).map(Module::id), Some("staying"));
    }

    #[test]
    fn hidden_modules_are_not_drawn_or_hit_tested() {
        let mut registry = ModuleRegistry::new();
        let mut hidden = RecordingModule::new("hidden", rect(0, 0, 4, 4));
        hidden.set_hidden(true);
        registry.register(Box::new(hidden));
        registry.register(Box::new(RecordingModule::new("visible", rect(0, 0, 4, 4))));

        let ids: Vec<&str> = registry.iter().map(Module::id).collect();
        assert_eq!(ids, vec!["visible"]);
        assert_eq!(registry.hit_test(1, 1).map(Module::id), Some("visible"));
    }

    #[test]
    fn update_hover_at_delivers_enter_to_the_topmost_module_under_the_point() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("a", rect(0, 0, 10, 10))));

        let hovered = registry.update_hover_at(Some((5, 5)));

        assert_eq!(hovered, Some("a"));
    }

    #[test]
    fn moving_the_hover_point_off_a_module_delivers_leave() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("a", rect(0, 0, 4, 4))));

        registry.update_hover_at(Some((1, 1)));
        let hovered = registry.update_hover_at(Some((99, 99)));

        assert_eq!(hovered, None);
    }

    #[test]
    fn update_hover_at_is_idempotent_while_the_point_stays_over_the_same_module() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("a", rect(0, 0, 10, 10))));

        registry.update_hover_at(Some((1, 1)));
        let hovered = registry.update_hover_at(Some((2, 2)));

        assert_eq!(hovered, Some("a"));
    }

    #[test]
    fn update_hover_at_moves_hover_between_two_modules() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(RecordingModule::new("a", rect(0, 0, 4, 4))));
        registry.register(Box::new(RecordingModule::new("b", rect(10, 10, 14, 14))));

        registry.update_hover_at(Some((1, 1)));
        let hovered = registry.update_hover_at(Some((11, 11)));

        assert_eq!(hovered, Some("b"));
    }
}
