use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight,
    Hotspot, ModulePointerButton, ModulePointerEvent, ModuleRect, Scrollbar, ScrollbarSpan,
    UiColorRole, UiPalette, WorldPoint,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBarButton {
    pub id: String,
    pub label: String,
    /// Optional idle color. Omitted preserves the command bar's standard
    /// Medium presentation.
    pub idle_color_role: Option<UiColorRole>,
    /// Optional shared-tooltip content for this button. The command bar only
    /// declares its precise hit rect; dwell and card rendering stay shared.
    pub tooltip: Option<CommandBarButtonTooltip>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBarButtonTooltip {
    pub title: String,
    pub description: String,
}

impl CommandBarButton {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            idle_color_role: None,
            tooltip: None,
        }
    }

    pub fn with_idle_color_role(mut self, color_role: UiColorRole) -> Self {
        self.idle_color_role = Some(color_role);
        self
    }

    pub fn with_tooltip(
        mut self,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.tooltip = Some(CommandBarButtonTooltip {
            title: title.into(),
            description: description.into(),
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBarClickOutcome {
    ExpandToggled { expanded: bool },
    SeamlessToggled { seamless: bool },
    ButtonPressed { button_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedCommandBarState {
    pub expanded: bool,
    pub seamless: bool,
    #[serde(default)]
    pub menu_path: Vec<String>,
    #[serde(default)]
    pub scroll_offset: usize,
}

impl Default for PersistedCommandBarState {
    fn default() -> Self {
        Self {
            expanded: false,
            seamless: false,
            menu_path: Vec::new(),
            scroll_offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBarLayout {
    pub inset_left: i32,
    pub inset_right: i32,
    pub inset_bottom: i32,
    pub expanded_height: i32,
    pub expanded_underhang: i32,
}

impl Default for CommandBarLayout {
    fn default() -> Self {
        Self {
            inset_left: 2,
            inset_right: 2,
            inset_bottom: 1,
            expanded_height: 4,
            expanded_underhang: 3,
        }
    }
}

pub struct CommandBar {
    id: String,
    palette: UiPalette,
    layout: CommandBarLayout,
    buttons: Vec<CommandBarButton>,
    nested_buttons: BTreeMap<String, Vec<CommandBarButton>>,
    expanded: bool,
    seamless: bool,
    hovered: bool,
    expanded_rect: ModuleRect,
    menu_path: Vec<String>,
    scroll_offset: usize,
    hovered_button_id: Option<String>,
    /// Horizontal scrollbar over the button list, shown when the buttons
    /// overflow the bar. Span lives on the row just below the button row,
    /// re-synced on every layout update; drag state survives re-placement.
    scrollbar: Scrollbar,
}

impl CommandBar {
    pub fn new(id: impl Into<String>, palette: UiPalette) -> Self {
        Self {
            id: id.into(),
            palette,
            layout: CommandBarLayout::default(),
            buttons: Vec::new(),
            nested_buttons: BTreeMap::new(),
            expanded: false,
            seamless: false,
            hovered: false,
            expanded_rect: ModuleRect {
                x0: -2,
                y0: -2,
                x1: 2,
                y1: 1,
            },
            menu_path: Vec::new(),
            scroll_offset: 0,
            hovered_button_id: None,
            scrollbar: Scrollbar::new(ScrollbarSpan::Horizontal { y: 0, x0: 0, x1: 0 }),
        }
    }

    pub fn with_layout(mut self, layout: CommandBarLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_buttons(mut self, buttons: Vec<CommandBarButton>) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn with_nested_buttons(
        mut self,
        button_id: impl Into<String>,
        buttons: Vec<CommandBarButton>,
    ) -> Self {
        self.nested_buttons.insert(button_id.into(), buttons);
        self
    }

    pub fn set_buttons(&mut self, buttons: Vec<CommandBarButton>) {
        self.buttons = buttons;
        self.clamp_scroll_offset();
    }

    pub fn set_nested_buttons(
        &mut self,
        button_id: impl Into<String>,
        buttons: Vec<CommandBarButton>,
    ) {
        self.nested_buttons.insert(button_id.into(), buttons);
        self.clamp_scroll_offset();
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn is_seamless(&self) -> bool {
        self.seamless
    }

    pub fn persisted_state(&self) -> PersistedCommandBarState {
        PersistedCommandBarState {
            expanded: self.expanded,
            seamless: self.seamless,
            menu_path: self.menu_path.clone(),
            scroll_offset: self.scroll_offset,
        }
    }

    pub fn apply_persisted_state(&mut self, state: &PersistedCommandBarState) {
        self.expanded = state.expanded;
        self.seamless = state.seamless;
        self.menu_path = state.menu_path.clone();
        self.scroll_offset = state.scroll_offset;
        self.clamp_menu_path();
        self.clamp_scroll_offset();
    }

    pub fn set_pointer_position(&mut self, pointer: Option<(i32, i32)>) {
        self.hovered = pointer.is_some_and(|(x, y)| self.contains(x, y));
        self.hovered_button_id = pointer.and_then(|(x, y)| self.button_id_at(x, y));
    }

    pub fn update_layout(&mut self, cell_clip_size: [f32; 2], hud_pan_offset: CellPoint) {
        let screen = fully_visible_screen_camera_rect(cell_clip_size);
        let width =
            (screen.x1 - screen.x0 + 1 - self.layout.inset_left - self.layout.inset_right).max(1);
        let base_x0 = screen.x0 + self.layout.inset_left - hud_pan_offset.x;
        let base_y0 = screen.y0 + self.layout.inset_bottom - hud_pan_offset.y;
        let underhang = self.layout.expanded_underhang.max(0);
        let expanded_height = self.layout.expanded_height.max(1);
        self.expanded_rect = ModuleRect {
            x0: base_x0,
            y0: base_y0 - underhang,
            x1: base_x0 + width - 1,
            y1: base_y0 + expanded_height - 1,
        };
        // Scrollbar re-placement on every resize: one row below the
        // button row (two rows below the top border), spanning the button
        // region. Drag state survives the re-placement.
        let height = self.expanded_rect.y1 - self.expanded_rect.y0;
        self.scrollbar.set_span(ScrollbarSpan::Horizontal {
            y: height - 2,
            x0: 3,
            x1: width - 1,
        });
    }

    pub fn rect(&self) -> ModuleRect {
        if self.shows_expanded_body() {
            self.expanded_rect
        } else {
            let (x, y) = self.expand_handle_position();
            ModuleRect {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            }
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.rect().contains(x, y)
    }

    /// Declares tooltip hotspots for visible buttons that explicitly opted in.
    /// Their rects share `button_layout` with click handling, so a tooltip
    /// cannot describe an off-screen or non-clickable button.
    pub fn hotspots(&self) -> Vec<Hotspot> {
        let button_y = self.button_row_y();
        self.button_layout()
            .into_iter()
            .filter_map(|entry| {
                let tooltip = entry.button.tooltip?;
                let width = entry.button.label.chars().count() as i32;
                Some(Hotspot::new(
                    ModuleRect {
                        x0: entry.x0,
                        y0: button_y,
                        x1: entry.x0 + width - 1,
                        y1: button_y,
                    },
                    tooltip.title,
                    tooltip.description,
                ))
            })
            .collect()
    }

    pub fn draw(&self) -> CellGroup {
        let rect = self.rect();
        let origin = WorldPoint {
            x: rect.x0,
            y: rect.y0,
            z: 0,
        };
        let mut cells = Vec::new();

        if self.shows_expanded_body() {
            let width = rect.x1 - rect.x0;
            let height = rect.y1 - rect.y0;
            let background = self.palette.get(UiColorRole::Background);
            let border = self.palette.get(UiColorRole::Dimmest);
            let control = self.palette.get(UiColorRole::Vivid);
            let idle = self.palette.get(UiColorRole::Medium);

            for y in 0..height {
                for x in 0..=width {
                    cells.push(command_bar_cell(x, y, ' ', background, 1));
                }
            }
            for x in 0..=width {
                cells.push(command_bar_cell(x, height, '━', border, 2));
            }
            let handle_x = self.expand_handle_position().0 - rect.x0;
            cells.push(command_bar_cell(handle_x, height, 'v', control, 3));
            cells.push(command_bar_cell(1, height, 'S', idle, 2));

            let button_y = self.button_row_y() - rect.y0;
            for entry in self.button_layout() {
                let is_hovered =
                    self.hovered_button_id.as_deref() == Some(entry.button.id.as_str());
                let button_color = if is_hovered {
                    control
                } else {
                    self.palette
                        .get(entry.button.idle_color_role.unwrap_or(UiColorRole::Medium))
                };
                let button_weight = if is_hovered { 2 } else { 1 };
                for (index, glyph) in entry.button.label.chars().enumerate() {
                    let x = entry.x0 - rect.x0 + index as i32;
                    if x > width {
                        break;
                    }
                    cells.push(command_bar_cell(
                        x,
                        button_y,
                        glyph,
                        button_color,
                        button_weight,
                    ));
                }
            }
            // Overflow scrollbar under the button row: only when the
            // button list does not fit.
            let content = self.active_buttons().len();
            let viewport = self.button_viewport_capacity();
            for cell in self.scrollbar.cells(
                content,
                viewport,
                self.scroll_offset,
                &self.palette,
                None,
                self.scrollbar.is_active(),
            )
            {
                cells.push(cell);
            }
        } else {
            let color = self.palette.get(UiColorRole::Vivid);
            cells.push(command_bar_cell(0, 0, '^', color, 3));
        }

        CellGroup::from_cells(origin, cells).with_intake_behavior(CellGroupIntakeBehavior::Flat2d)
    }

    pub fn on_pointer_event(
        &mut self,
        event: ModulePointerEvent,
    ) -> Option<CommandBarClickOutcome> {
        match event {
            ModulePointerEvent::Enter => None,
            ModulePointerEvent::Leave => {
                self.hovered = false;
                self.hovered_button_id = None;
                None
            }
            ModulePointerEvent::Move { x, y } => {
                self.set_pointer_position(Some((x, y)));
                None
            }
            ModulePointerEvent::Click {
                x,
                y,
                button: ModulePointerButton::Left,
            } => {
                // A click on the scrollbar row is scrollbar interaction:
                // thumb grab or track page-jump, never a button press.
                // Pointer events arrive in screen space; the scrollbar span
                // is module-local, so convert before hit-testing.
                let content = self.active_buttons().len();
                let viewport = self.button_viewport_capacity();
                if Scrollbar::is_visible(content, viewport) {
                    let rect = self.expanded_rect;
                    let (local_x, local_y) = (x - rect.x0, y - rect.y0);
                    if self.scrollbar.contains(local_x, local_y) {
                        self.scrollbar.on_down(
                            local_x,
                            local_y,
                            content,
                            viewport,
                            &mut self.scroll_offset,
                        );
                        return None;
                    }
                }
                self.handle_left_click(x, y)
            }
            ModulePointerEvent::Down { .. }
            | ModulePointerEvent::Up { .. }
            | ModulePointerEvent::Click { .. } => None,
        }
    }

    /// Whether a scrollbar thumb drag is live: the host feeds pointer moves
    /// and the eventual release while this is true.
    pub fn is_scrollbar_dragging(&self) -> bool {
        self.scrollbar.is_dragging()
    }

    /// Pointer move during a live scrollbar drag: re-maps the grab point to
    /// a new scroll offset. Returns whether the offset changed. Screen-space
    /// pointer coords are converted to the bar's module-local span.
    pub fn drag_scrollbar(&mut self, x: i32, y: i32) -> bool {
        let content = self.active_buttons().len();
        let viewport = self.button_viewport_capacity();
        let rect = self.expanded_rect;
        self.scrollbar.on_drag(
            x - rect.x0,
            y - rect.y0,
            content,
            viewport,
            &mut self.scroll_offset,
        )
    }

    /// Pointer release: ends the scrollbar drag session.
    pub fn end_scrollbar_drag(&mut self) {
        self.scrollbar.on_up();
    }

    pub fn on_wheel(&mut self, x: i32, y: i32, delta_x: f32, delta_y: f32) -> bool {
        if !self.shows_expanded_body() || !self.contains(x, y) {
            return false;
        }
        // Single horizontal bar rule (J): vertical wheel scrolls it too; a
        // mouse with a horizontal wheel scrolls it directly.
        let steps = Scrollbar::wheel_steps(delta_x, delta_y, self.scrollbar.span().axis());
        if steps == 0.0 {
            return false;
        }
        if steps > 0.0 {
            self.scroll_right();
        } else {
            self.scroll_left();
        }
        true
    }

    fn shows_expanded_body(&self) -> bool {
        self.expanded && (!self.seamless || self.hovered)
    }

    fn expand_handle_position(&self) -> (i32, i32) {
        (
            self.expanded_rect.x0 + (self.expanded_rect.x1 - self.expanded_rect.x0) / 2,
            if self.shows_expanded_body() {
                self.expanded_rect.y1
            } else {
                self.expanded_rect.y0 + self.layout.expanded_underhang.max(0)
            },
        )
    }

    fn silence_position(&self) -> Option<(i32, i32)> {
        self.shows_expanded_body()
            .then_some((self.expanded_rect.x0 + 1, self.expanded_rect.y1))
    }

    fn button_row_y(&self) -> i32 {
        (self.expanded_rect.y1 - 1).max(self.expanded_rect.y0)
    }

    fn button_layout(&self) -> Vec<ButtonLayoutEntry> {
        if !self.shows_expanded_body() {
            return Vec::new();
        }
        let mut layout = Vec::new();
        let mut x = self.expanded_rect.x0 + 3;
        let limit = self.expanded_rect.x1 - 1;
        for button in self.active_buttons().into_iter().skip(self.scroll_offset) {
            let width = button.label.chars().count() as i32;
            if x + width - 1 > limit {
                break;
            }
            layout.push(ButtonLayoutEntry { button, x0: x });
            x += width + 2;
        }
        layout
    }

    fn active_buttons(&self) -> Vec<CommandBarButton> {
        let mut buttons = Vec::new();
        if !self.menu_path.is_empty() {
            buttons.push(back_button());
        }
        let source = self
            .menu_path
            .last()
            .and_then(|id| self.nested_buttons.get(id))
            .unwrap_or(&self.buttons);
        buttons.extend(source.iter().cloned());
        buttons
    }

    fn clamp_menu_path(&mut self) {
        while self
            .menu_path
            .last()
            .is_some_and(|id| !self.nested_buttons.contains_key(id))
        {
            self.menu_path.pop();
        }
    }

    /// How many buttons fit the button row from the start of the list: the
    /// button-unit viewport the scrollbar and clamp math share.
    fn button_viewport_capacity(&self) -> usize {
        if !self.shows_expanded_body() {
            return 0;
        }
        let limit = self.expanded_rect.x1 - 1;
        let mut x = self.expanded_rect.x0 + 3;
        let mut count = 0;
        for button in self.active_buttons() {
            let width = button.label.chars().count() as i32;
            if x + width - 1 > limit {
                break;
            }
            x += width + 2;
            count += 1;
        }
        count
    }

    /// Scroll range over the button list: how many buttons sit past what
    /// fits. Collapsed bars keep the legacy `count - 1` guard so a persisted
    /// offset stays sane until the next expanded layout update.
    fn scroll_max(&self) -> usize {
        let button_count = self.active_buttons().len();
        let capacity = self.button_viewport_capacity();
        if capacity > 0 {
            button_count.saturating_sub(capacity)
        } else {
            button_count.saturating_sub(1)
        }
    }

    fn clamp_scroll_offset(&mut self) {
        self.scroll_offset = self.scroll_offset.min(self.scroll_max());
    }

    fn scroll_right(&mut self) {
        self.scroll_offset = (self.scroll_offset + 1).min(self.scroll_max());
    }

    fn scroll_left(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    fn button_id_at(&self, x: i32, y: i32) -> Option<String> {
        if y != self.button_row_y() {
            return None;
        }
        for entry in self.button_layout() {
            let width = entry.button.label.chars().count() as i32;
            if x >= entry.x0 && x < entry.x0 + width {
                return Some(entry.button.id);
            }
        }
        None
    }

    fn handle_left_click(&mut self, x: i32, y: i32) -> Option<CommandBarClickOutcome> {
        let handle = self.expand_handle_position();
        if (x, y) == handle {
            self.expanded = !self.expanded;
            return Some(CommandBarClickOutcome::ExpandToggled {
                expanded: self.expanded,
            });
        }
        if let Some(silence) = self.silence_position() {
            if (x, y) == silence {
                self.seamless = !self.seamless;
                return Some(CommandBarClickOutcome::SeamlessToggled {
                    seamless: self.seamless,
                });
            }
        }
        if let Some(button_id) = self.button_id_at(x, y) {
            if button_id == back_button().id {
                self.menu_path.pop();
                self.scroll_offset = 0;
                return None;
            }
            if self.nested_buttons.contains_key(&button_id) {
                self.menu_path.push(button_id);
                self.scroll_offset = 0;
                self.clamp_scroll_offset();
                return None;
            }
            return Some(CommandBarClickOutcome::ButtonPressed { button_id });
        }
        None
    }
}

fn back_button() -> CommandBarButton {
    CommandBarButton::new("__command_bar_back__", "BACK")
}

struct ButtonLayoutEntry {
    button: CommandBarButton,
    x0: i32,
}

fn command_bar_cell(x: i32, y: i32, glyph: char, color: CellColor, weight_index: i32) -> Cell {
    Cell {
        position: CellPoint { x, y, z: 0 },
        graphic: CellGraphic::Glyph(glyph),
        color,
        weight: CellWeight::from_index_clamped(weight_index),
        ..Cell::default()
    }
}

fn fully_visible_screen_camera_rect(cell_clip_size: [f32; 2]) -> ModuleRect {
    let x_half = cell_clip_size[0].abs() * 0.5;
    let y_half = cell_clip_size[1].abs() * 0.5;
    ModuleRect {
        x0: ((-1.0 + x_half) / cell_clip_size[0]).ceil() as i32,
        y0: ((-1.0 + y_half) / cell_clip_size[1]).ceil() as i32,
        x1: ((1.0 - x_half) / cell_clip_size[0]).floor() as i32,
        y1: ((1.0 - y_half) / cell_clip_size[1]).floor() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_at(group: &CellGroup, x: i32, y: i32) -> Option<char> {
        group.cells.get(&CellPoint { x, y, z: 0 }).and_then(|cell| {
            if let CellGraphic::Glyph(glyph) = cell.graphic {
                Some(glyph)
            } else {
                None
            }
        })
    }

    #[test]
    fn collapsed_bar_draws_a_single_centered_expand_handle() {
        let mut bar = CommandBar::new("bar", UiPalette::default());
        bar.update_layout([0.2, 0.2], CellPoint::origin());

        let rect = bar.rect();
        assert_eq!(
            rect,
            ModuleRect {
                x0: 0,
                y0: -3,
                x1: 0,
                y1: -3
            }
        );
        let group = bar.draw();
        assert_eq!(group.cells.len(), 1);
        assert_eq!(glyph_at(&group, 0, 0), Some('^'));
    }

    #[test]
    fn expanded_bar_stays_screen_locked_even_when_hud_pans() {
        let mut bar = CommandBar::new("bar", UiPalette::default());
        bar.update_layout([0.2, 0.2], CellPoint { x: 3, y: -2, z: 0 });
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: -3,
            y: -1,
            button: ModulePointerButton::Left,
        });
        bar.set_pointer_position(Some((-3, -1)));

        assert_eq!(
            bar.rect(),
            ModuleRect {
                x0: -5,
                y0: -4,
                x1: -1,
                y1: 2,
            }
        );
    }

    #[test]
    fn expanded_bar_draws_only_a_top_border_and_controls() {
        let mut bar = CommandBar::new("bar", UiPalette::default());
        bar.update_layout([0.1, 0.1], CellPoint::origin());
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: 0,
            y: -8,
            button: ModulePointerButton::Left,
        });
        bar.set_pointer_position(Some((0, -8)));

        let group = bar.draw();
        assert_eq!(glyph_at(&group, 0, 6), Some('━'));
        assert_eq!(glyph_at(&group, 7, 6), Some('v'));
        assert_eq!(glyph_at(&group, 1, 6), Some('S'));
        assert_eq!(glyph_at(&group, 0, 0), Some(' '));
        assert_eq!(glyph_at(&group, 0, 5), Some(' '));
        assert_eq!(glyph_at(&group, 14, 0), Some(' '));
        assert_eq!(glyph_at(&group, 14, 5), Some(' '));
    }

    #[test]
    fn clicking_the_handle_toggles_expansion() {
        let mut bar = CommandBar::new("bar", UiPalette::default());
        bar.update_layout([0.2, 0.2], CellPoint::origin());

        assert_eq!(
            bar.on_pointer_event(ModulePointerEvent::Click {
                x: 0,
                y: -3,
                button: ModulePointerButton::Left,
            }),
            Some(CommandBarClickOutcome::ExpandToggled { expanded: true })
        );
        bar.set_pointer_position(Some((0, -3)));
        assert_eq!(bar.rect().y1, 0);
    }

    #[test]
    fn clicking_an_expanded_text_button_reports_its_id() {
        let mut bar = CommandBar::new("bar", UiPalette::default())
            .with_buttons(vec![CommandBarButton::new("modules", "MODULES")]);
        bar.update_layout([0.1, 0.1], CellPoint::origin());
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: 0,
            y: -8,
            button: ModulePointerButton::Left,
        });
        bar.set_pointer_position(Some((0, -8)));

        let button = bar.button_layout().into_iter().next().unwrap();
        assert_eq!(
            bar.on_pointer_event(ModulePointerEvent::Click {
                x: button.x0,
                y: bar.button_row_y(),
                button: ModulePointerButton::Left,
            }),
            Some(CommandBarClickOutcome::ButtonPressed {
                button_id: "modules".to_string(),
            })
        );
    }

    #[test]
    fn expanded_buttons_draw_inside_the_bar_when_hud_panned() {
        let mut bar = CommandBar::new("bar", UiPalette::default())
            .with_buttons(vec![CommandBarButton::new("modules", "MODULES")]);
        bar.update_layout([0.1, 0.1], CellPoint { x: 3, y: -2, z: 0 });
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: -3,
            y: -6,
            button: ModulePointerButton::Left,
        });
        bar.set_pointer_position(Some((-3, -6)));

        let group = bar.draw();
        assert_eq!(glyph_at(&group, 3, 5), Some('M'));
        assert_eq!(glyph_at(&group, 3, 0), Some(' '));
    }

    #[test]
    fn configured_button_uses_its_idle_palette_role_without_changing_defaults() {
        let palette = UiPalette::default();
        let mut bar = CommandBar::new("bar", palette.clone()).with_buttons(vec![
            CommandBarButton::new("default", "DEFAULT"),
            CommandBarButton::new("status", "STATUS").with_idle_color_role(UiColorRole::Bright),
        ]);
        bar.update_layout([0.05, 0.1], CellPoint::origin());
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: 0,
            y: -8,
            button: ModulePointerButton::Left,
        });

        let entries = bar.button_layout();
        let group = bar.draw();
        let default_cell = group
            .cells
            .get(&CellPoint {
                x: entries[0].x0 - bar.rect().x0,
                y: bar.button_row_y() - bar.rect().y0,
                z: 0,
            })
            .unwrap();
        let status_cell = group
            .cells
            .get(&CellPoint {
                x: entries[1].x0 - bar.rect().x0,
                y: bar.button_row_y() - bar.rect().y0,
                z: 0,
            })
            .unwrap();
        assert_eq!(default_cell.color, palette.get(UiColorRole::Medium));
        assert_eq!(status_cell.color, palette.get(UiColorRole::Bright));
    }

    #[test]
    fn hotspots_follow_visible_tooltipped_button_layout() {
        let mut bar = CommandBar::new("bar", UiPalette::default()).with_buttons(vec![
            CommandBarButton::new("file", "FILE").with_tooltip("File", "Create or save."),
            CommandBarButton::new("plain", "PLAIN"),
        ]);
        bar.update_layout([0.1, 0.1], CellPoint::origin());
        assert!(bar.hotspots().is_empty());

        bar.on_pointer_event(ModulePointerEvent::Click {
            x: 0,
            y: -8,
            button: ModulePointerButton::Left,
        });
        let entry = bar.button_layout().into_iter().next().unwrap();
        assert_eq!(
            bar.hotspots(),
            vec![Hotspot::new(
                ModuleRect {
                    x0: entry.x0,
                    y0: bar.button_row_y(),
                    x1: entry.x0 + 3,
                    y1: bar.button_row_y(),
                },
                "File",
                "Create or save.",
            )]
        );
    }

    /// An expanded bar whose five 5-char buttons do not all fit, so the
    /// scrollbar shows. Four fit the button row; one sits past the end.
    fn overflow_bar() -> CommandBar {
        let mut bar = CommandBar::new("bar", UiPalette::default()).with_buttons(vec![
            CommandBarButton::new("a", "AAAAA"),
            CommandBarButton::new("b", "BBBBB"),
            CommandBarButton::new("c", "CCCCC"),
            CommandBarButton::new("d", "DDDDD"),
            CommandBarButton::new("e", "EEEEE"),
        ]);
        bar.update_layout([0.05, 0.1], CellPoint::origin());
        let (handle_x, handle_y) = bar.expand_handle_position();
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: handle_x,
            y: handle_y,
            button: ModulePointerButton::Left,
        });
        bar
    }

    #[test]
    fn overflowing_buttons_draw_a_scrollbar_below_the_button_row() {
        let bar = overflow_bar();
        let rect = bar.rect();
        let width = rect.x1 - rect.x0;
        let height = rect.y1 - rect.y0;
        let group = bar.draw();
        // Offset 0: the thumb leads the track (local draw coords); the
        // occupied left end cap keeps its own glyph, weight boosted.
        assert_eq!(glyph_at(&group, 3, height - 2), Some('┣'));
        assert_eq!(glyph_at(&group, width - 1, height - 2), Some('═'));
    }

    #[test]
    fn fitting_buttons_draw_no_scrollbar() {
        let mut bar = CommandBar::new("bar", UiPalette::default()).with_buttons(vec![
            CommandBarButton::new("a", "AAAA"),
            CommandBarButton::new("b", "BBBB"),
        ]);
        bar.update_layout([0.05, 0.1], CellPoint::origin());
        let (handle_x, handle_y) = bar.expand_handle_position();
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: handle_x,
            y: handle_y,
            button: ModulePointerButton::Left,
        });
        let local_bar_y = bar.rect().y1 - bar.rect().y0 - 2;
        let local_width = bar.rect().x1 - bar.rect().x0;
        let group = bar.draw();
        // The row only carries the plain background: no scrollbar cells.
        assert_eq!(glyph_at(&group, local_width - 1, local_bar_y), Some(' '));
    }

    #[test]
    fn track_click_page_jumps_then_drags_and_releases() {
        let mut bar = overflow_bar();
        let rect = bar.rect();
        let bar_y = rect.y1 - 2;
        // Content 5, viewport 4: max offset 1.
        // Click the track's far end: jump the thumb there, drag session on.
        assert_eq!(bar.scroll_offset, 0);
        assert_eq!(
            bar.on_pointer_event(ModulePointerEvent::Click {
                x: rect.x1 - 1,
                y: bar_y,
                button: ModulePointerButton::Left,
            }),
            None,
            "scrollbar clicks never fall through to button presses"
        );
        assert_eq!(bar.scroll_offset, 1);
        assert!(bar.is_scrollbar_dragging());
        // Drag back to the track start, then release.
        assert!(bar.drag_scrollbar(rect.x0 + 3, bar_y));
        assert_eq!(bar.scroll_offset, 0);
        bar.end_scrollbar_drag();
        assert!(!bar.is_scrollbar_dragging());
    }

    #[test]
    fn thumb_drag_follows_the_pointer_until_release() {
        let mut bar = overflow_bar();
        let rect = bar.rect();
        let bar_y = rect.y1 - 2;
        // Grab the thumb's leading edge.
        bar.on_pointer_event(ModulePointerEvent::Click {
            x: rect.x0 + 3,
            y: bar_y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(bar.scroll_offset, 0);
        assert!(bar.is_scrollbar_dragging());
        assert!(bar.drag_scrollbar(rect.x1 - 1, bar_y));
        assert_eq!(bar.scroll_offset, 1);
        bar.end_scrollbar_drag();
        assert!(!bar.is_scrollbar_dragging());
    }

    #[test]
    fn either_wheel_axis_scrolls_the_horizontal_bar() {
        let mut bar = overflow_bar();
        let rect = bar.rect();
        let bar_y = rect.y1 - 2;
        // Vertical wheel down scrolls toward the end (single-bar rule).
        assert!(bar.on_wheel(rect.x0 + 10, bar_y, 0.0, -1.0));
        assert_eq!(bar.scroll_offset, 1);
        // Clamped at the end.
        assert!(bar.on_wheel(rect.x0 + 10, bar_y, 0.0, -1.0));
        assert_eq!(bar.scroll_offset, 1);
        // Horizontal wheel back toward the start.
        assert!(bar.on_wheel(rect.x0 + 10, bar_y, -1.0, 0.0));
        assert_eq!(bar.scroll_offset, 0);
        // Horizontal wheel toward the end again.
        assert!(bar.on_wheel(rect.x0 + 10, bar_y, 1.0, 0.0));
        assert_eq!(bar.scroll_offset, 1);
    }
}
