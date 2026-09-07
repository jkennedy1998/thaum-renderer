use thaum_renderer_tools_color::nearest_rgb_in_collection;

use crate::{
    Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight,
    GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, Hotspot, Module, ModulePointerEvent,
    ModuleRect, PanelChrome, PersistedModuleUiState, UiColorRole, UiPalette, WorldPoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragTarget {
    Field,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HsvColor {
    hue: f32,
    saturation: f32,
    value: f32,
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn rgb_to_hsv(rgb: [u8; 3]) -> HsvColor {
    let red = rgb[0] as f32 / 255.0;
    let green = rgb[1] as f32 / 255.0;
    let blue = rgb[2] as f32 / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;

    let hue = if delta <= f32::EPSILON {
        0.0
    } else if (max - red).abs() <= f32::EPSILON {
        (((green - blue) / delta) / 6.0).rem_euclid(1.0)
    } else if (max - green).abs() <= f32::EPSILON {
        (((blue - red) / delta) + 2.0) / 6.0
    } else {
        (((red - green) / delta) + 4.0) / 6.0
    };

    HsvColor {
        hue,
        saturation: if max <= f32::EPSILON {
            0.0
        } else {
            delta / max
        },
        value: max,
    }
}

fn hsv_to_rgb(hsv: HsvColor) -> [u8; 3] {
    let hue = hsv.hue.rem_euclid(1.0);
    let saturation = clamp01(hsv.saturation);
    let value = clamp01(hsv.value);
    let chroma = value * saturation;
    let scaled_hue = hue * 6.0;
    let secondary = chroma * (1.0 - ((scaled_hue.rem_euclid(2.0)) - 1.0).abs());
    let (red, green, blue) = match scaled_hue.floor() as i32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let match_value = value - chroma;
    [
        ((red + match_value) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((green + match_value) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((blue + match_value) * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn content_layout(rect: ModuleRect) -> (i32, i32, i32, i32, i32) {
    let (content_x, content_y) = PanelChrome::content_origin();
    let (content_width, content_height) = PanelChrome::content_size(rect);
    let field_x0 = content_x;
    let field_x1 = (content_x + content_width - 3).max(content_x);
    let field_y0 = content_y + 2;
    let field_y1 = (content_y + content_height - 1).max(field_y0);
    let slider_x = (field_x1 + 2).min(content_x + content_width - 1);
    (field_x0, field_x1, field_y0, field_y1, slider_x)
}

fn ratio_at(position: i32, start: i32, end: i32) -> f32 {
    if end <= start {
        0.0
    } else {
        (position - start) as f32 / (end - start) as f32
    }
    .clamp(0.0, 1.0)
}

const WHEEL_HUE_STEP: f32 = 1.0 / 36.0;

fn rgb_cell(rgb: [u8; 3]) -> CellColor {
    CellColor::Flat([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    ])
}

pub struct ColorBlockModule {
    id: String,
    rect: ModuleRect,
    palette: UiPalette,
    indexed_palette: Vec<[u8; 3]>,
    selected_rgb: [u8; 3],
    selected_hsv: HsvColor,
    drag_target: Option<DragTarget>,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
}

impl ColorBlockModule {
    pub fn new(id: impl Into<String>, rect: ModuleRect, palette: UiPalette) -> Self {
        let selected_rgb = [235, 235, 235];
        Self {
            id: id.into(),
            rect,
            palette,
            indexed_palette: Vec::new(),
            selected_rgb,
            selected_hsv: rgb_to_hsv(selected_rgb),
            drag_target: None,
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            hidden: false,
        }
    }

    pub fn with_indexed_palette(mut self, indexed_palette: &[[u8; 3]]) -> Self {
        self.indexed_palette = indexed_palette.to_vec();
        self.selected_rgb = self.resolve_rgb(self.selected_hsv);
        self
    }

    pub fn selected_rgb(&self) -> [u8; 3] {
        self.selected_rgb
    }

    pub fn set_selected_rgb(&mut self, rgb: [u8; 3]) {
        let snapped = self.snap_rgb(rgb);
        if snapped == self.selected_rgb {
            // Re-applying the already-committed color (e.g. a consumer syncing
            // its hand color before a click) must not re-derive HSV from the
            // snapped palette entry: that would drag the wheel-scrolled hue
            // marker away from where the user left it.
            return;
        }
        self.selected_rgb = snapped;
        self.selected_hsv = rgb_to_hsv(snapped);
    }

    fn snap_rgb(&self, rgb: [u8; 3]) -> [u8; 3] {
        if self.indexed_palette.is_empty() {
            rgb
        } else {
            self.indexed_palette[nearest_rgb_in_collection(rgb, &self.indexed_palette)]
        }
    }

    fn resolve_rgb(&self, hsv: HsvColor) -> [u8; 3] {
        self.snap_rgb(hsv_to_rgb(hsv))
    }

    fn set_from_field_point(&mut self, x: i32, y: i32) {
        let (field_x0, field_x1, field_y0, field_y1, _) = content_layout(self.rect);
        self.selected_hsv.saturation = ratio_at(x, field_x0, field_x1);
        self.selected_hsv.value = ratio_at(y, field_y0, field_y1);
        self.selected_rgb = self.resolve_rgb(self.selected_hsv);
    }

    fn scroll_hue(&mut self, delta_y: f32) {
        if delta_y == 0.0 {
            return;
        }
        self.selected_hsv.hue = (self.selected_hsv.hue
            + if delta_y > 0.0 {
                WHEEL_HUE_STEP
            } else {
                -WHEEL_HUE_STEP
            })
        .rem_euclid(1.0);
        self.selected_rgb = self.resolve_rgb(self.selected_hsv);
    }
}

impl Module for ColorBlockModule {
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
        let (field_x0, field_x1, field_y0, field_y1, slider_x) = content_layout(self.rect);
        let mut cells: Vec<Cell> = if self.gizmo_state.is_seamless() {
            Vec::new()
        } else {
            self.gizmo_state
                .decorate_panel_chrome(
                    PanelChrome::new(self.rect, &self.palette)
                        .with_title("COLOR BLOCK")
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

        for y in field_y0..=field_y1 {
            for x in field_x0..=field_x1 {
                let rgb = self.resolve_rgb(HsvColor {
                    hue: self.selected_hsv.hue,
                    saturation: ratio_at(x, field_x0, field_x1),
                    value: ratio_at(y, field_y0, field_y1),
                });
                cells.push(Cell {
                    position: CellPoint { x, y, z: 0 },
                    graphic: CellGraphic::Glyph('█'),
                    color: rgb_cell(rgb),
                    weight: CellWeight::from_index_clamped(1),
                    ..Cell::default()
                });
            }
        }

        for y in field_y0..=field_y1 {
            let rgb = self.resolve_rgb(HsvColor {
                hue: ratio_at(y, field_y0, field_y1),
                saturation: 1.0,
                value: 1.0,
            });
            cells.push(Cell {
                position: CellPoint {
                    x: slider_x,
                    y,
                    z: 0,
                },
                graphic: CellGraphic::Glyph('█'),
                color: rgb_cell(rgb),
                weight: CellWeight::from_index_clamped(1),
                ..Cell::default()
            });
        }

        let marker_x =
            field_x0 + ((field_x1 - field_x0) as f32 * self.selected_hsv.saturation).round() as i32;
        let marker_y =
            field_y0 + ((field_y1 - field_y0) as f32 * self.selected_hsv.value).round() as i32;
        cells.push(Cell {
            position: CellPoint {
                x: marker_x.clamp(field_x0, field_x1),
                y: marker_y.clamp(field_y0, field_y1),
                z: 0,
            },
            graphic: CellGraphic::Glyph('◎'),
            color: self.palette.get(UiColorRole::Vivid),
            weight: CellWeight::from_index_clamped(3),
            ..Cell::default()
        });
        let slider_y =
            field_y0 + ((field_y1 - field_y0) as f32 * self.selected_hsv.hue).round() as i32;
        cells.push(Cell {
            position: CellPoint {
                x: slider_x - 1,
                y: slider_y.clamp(field_y0, field_y1),
                z: 0,
            },
            graphic: CellGraphic::Glyph('▶'),
            color: self.palette.get(UiColorRole::Vivid),
            weight: CellWeight::from_index_clamped(2),
            ..Cell::default()
        });

        let hex = format!(
            "#{:02X}{:02X}{:02X}",
            self.selected_rgb[0], self.selected_rgb[1], self.selected_rgb[2]
        );
        for (index, glyph) in hex.chars().enumerate() {
            cells.push(Cell {
                position: CellPoint {
                    x: field_x0 + index as i32,
                    y: field_y0 - 2,
                    z: 0,
                },
                graphic: CellGraphic::Glyph(glyph),
                color: self.palette.get(UiColorRole::Medium),
                weight: CellWeight::from_index_clamped(2),
                ..Cell::default()
            });
        }

        cells.push(Cell {
            position: CellPoint {
                x: slider_x,
                y: field_y0 - 2,
                z: 0,
            },
            graphic: CellGraphic::Glyph('█'),
            color: rgb_cell(self.selected_rgb),
            weight: CellWeight::from_index_clamped(3),
            ..Cell::default()
        });

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
                let (field_x0, field_x1, field_y0, field_y1, _) = content_layout(self.rect);
                if x >= self.rect.x0 + field_x0
                    && x <= self.rect.x0 + field_x1
                    && y >= self.rect.y0 + field_y0
                    && y <= self.rect.y0 + field_y1
                {
                    self.drag_target = Some(DragTarget::Field);
                    self.set_from_field_point(x - self.rect.x0, y - self.rect.y0);
                }
                // Clicks outside the field (hue-slider column included) are
                // intentionally inert: the hue marker may only move via wheel
                // scroll, so clicking can never relocate it.
            }
            ModulePointerEvent::Move { x, y } => {
                self.gizmo_state.note_pointer(&self.gizmos, self.rect, x, y);
                if let Some(next_rect) = self.gizmo_state.drag_rect(x, y) {
                    self.rect = next_rect;
                    return;
                }
                match self.drag_target {
                    Some(DragTarget::Field) => {
                        self.set_from_field_point(x - self.rect.x0, y - self.rect.y0)
                    }
                    None => {}
                }
            }
            ModulePointerEvent::Up { .. } => {
                self.drag_target = None;
                self.gizmo_state.end_drag();
            }
            ModulePointerEvent::Enter => self.gizmo_state.set_hovered(true),
            ModulePointerEvent::Leave => self.gizmo_state.set_hovered(false),
            ModulePointerEvent::Down { .. } => {}
        }
    }

    fn on_wheel(&mut self, _x: i32, _y: i32, _delta_x: f32, delta_y: f32) -> bool {
        self.scroll_hue(delta_y);
        true
    }

    fn wants_pointer_capture(&self) -> bool {
        self.gizmo_state.wants_pointer_capture() || self.drag_target.is_some()
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

    fn rect() -> ModuleRect {
        ModuleRect {
            x0: 0,
            y0: 0,
            x1: 18,
            y1: 11,
        }
    }

    #[test]
    fn clicking_the_field_selects_a_color() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default());
        let before = module.selected_rgb();

        module.on_pointer_event(ModulePointerEvent::Click {
            x: 4,
            y: 5,
            button: ModulePointerButton::Left,
        });

        assert_ne!(module.selected_rgb(), before);
    }

    #[test]
    fn clicking_the_slider_does_not_change_the_hue() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default());
        module.set_selected_rgb([255, 0, 0]);
        let before = module.selected_rgb();
        let hue_before = module.selected_hsv.hue;
        let (_, _, _, _, slider_x) = content_layout(rect());

        module.on_pointer_event(ModulePointerEvent::Click {
            x: slider_x,
            y: 5,
            button: ModulePointerButton::Left,
        });

        // The hue marker may only move via wheel scroll; a click on the
        // slider column must leave both the hue and the selection alone.
        assert_eq!(module.selected_hsv.hue, hue_before);
        assert_eq!(module.selected_rgb(), before);
    }

    #[test]
    fn reapplying_the_committed_rgb_keeps_the_scrolled_hue() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default())
            .with_indexed_palette(&[[0, 0, 0], [255, 0, 0], [255, 255, 255]]);
        module.selected_hsv.hue = 0.04;
        module.selected_rgb = module.resolve_rgb(module.selected_hsv);
        let committed = module.selected_rgb;
        let hue_before = module.selected_hsv.hue;

        // Consumers re-apply the committed hand color before each click;
        // that round-trip must not snap the continuous hue to the palette
        // entry's own hue.
        module.set_selected_rgb(committed);

        assert_eq!(module.selected_hsv.hue, hue_before);
    }

    #[test]
    fn clicking_the_field_starts_pointer_capture() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default());

        module.on_pointer_event(ModulePointerEvent::Click {
            x: 4,
            y: 5,
            button: ModulePointerButton::Left,
        });

        assert!(module.wants_pointer_capture());
    }

    #[test]
    fn dragging_updates_the_selected_color_until_pointer_up() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default());
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 4,
            y: 5,
            button: ModulePointerButton::Left,
        });
        let after_click = module.selected_rgb();

        module.on_pointer_event(ModulePointerEvent::Move { x: 10, y: 9 });
        assert_ne!(module.selected_rgb(), after_click);

        module.on_pointer_event(ModulePointerEvent::Up { x: 10, y: 9 });
        assert!(!module.wants_pointer_capture());
    }

    #[test]
    fn wheel_scroll_loops_the_hue() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default());
        module.set_selected_rgb([255, 0, 0]);

        module.on_wheel(0, 0, 0.0, 1.0);
        let after_up = module.selected_rgb();
        assert_ne!(after_up, [255, 0, 0]);

        for _ in 0..36 {
            module.on_wheel(0, 0, 0.0, -1.0);
        }
        assert_ne!(module.selected_rgb(), after_up);
    }

    #[test]
    fn indexed_palette_bands_the_field_output() {
        let mut module = ColorBlockModule::new("block", rect(), UiPalette::default())
            .with_indexed_palette(&[[0, 0, 0], [255, 0, 0], [255, 255, 255]]);

        module.on_pointer_event(ModulePointerEvent::Click {
            x: 4,
            y: 5,
            button: ModulePointerButton::Left,
        });

        assert!(matches!(
            module.selected_rgb(),
            [0, 0, 0] | [255, 0, 0] | [255, 255, 255]
        ));
    }
}
