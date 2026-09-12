use std::cell::{Cell as StdCell, RefCell};
use std::rc::Rc;

use crate::PerspectiveProfile;
use crate::{
    Cell, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight, GizmoBar,
    GizmoClickOutcome, GizmoKind, GizmoState, Hotspot, Module, title_hotspot, ModulePointerButton,
    ModulePointerEvent, ModuleRect, PanelChrome, ParallaxProfile, PersistedModuleUiState,
    PropertyRows, UiColorRole, UiPalette, WorldPoint,
};

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

/// Strength slider for mouse parallax. Kept small on purpose: parallax is a
/// slight drift, not a camera pan.
const PARALLAX_STRENGTH_KNOB: Knob = Knob {
    label: "str",
    presets: &[0.0, 0.05, 0.15, 0.25, 0.4, 0.5],
    wheel_step: 0.01,
    min: 0.0,
    max: ParallaxProfile::MAX_STRENGTH,
};

/// Rows below the perspective knobs: the parallax toggle, then the strength
/// slider. Both edit the shared [`ParallaxProfile`], not the perspective.
const PARALLAX_TOGGLE_ROW: usize = 3;
const PARALLAX_STRENGTH_ROW: usize = 4;
/// Depth row: the camera's render depth (focus-plane center). Wheel scrolls
/// the focus depth through the host-owned camera; click re-centers it at 0.
const DEPTH_ROW: usize = 5;
/// Bottom camera row: the camera's rendered-layer count (`visible_plane_radius` —
/// how many depth planes render on each side of the focus plane). Wheel
/// steps it through the host-owned camera; click cycles the presets.
pub const MAX_VISIBLE_PLANE_RADIUS: i32 = 512;
const LAYERS_ROW: usize = 6;
/// Zoom row: the camera's zoom — the same value the − and + keys drive
/// (`Camera::zoom_in`/`zoom_out`, one multiplicative step each). Wheel
/// steps one zoom step per notch (matching the wheel binding those keys
/// share); click cycles the zoom presets.
const ZOOM_ROW: usize = 7;
/// Presentation quality is an application preference rather than camera
/// projection truth, but belongs in this user-facing camera-settings panel.
/// It controls the percentage of the output surface used for offscreen
/// rendering before the renderer upscales it.
const QUALITY_ROW: usize = 8;
const ZOOM_PRESETS: &[f32] = &[0.05, 0.15, 0.3, 0.6, 1.0, 2.0, 4.0];
const LAYERS_PRESETS: &[i32] = &[
    0,
    1,
    2,
    4,
    8,
    12,
    16,
    24,
    32,
    64,
    128,
    256,
    MAX_VISIBLE_PLANE_RADIUS,
];
const ROW_COUNT: usize = 9;

/// User-selected offscreen render scale. Hosts apply this to their window
/// surface; it is deliberately independent from camera zoom and projection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderQualityProfile {
    internal_render_scale: f32,
}

impl Default for RenderQualityProfile {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl RenderQualityProfile {
    pub const MIN_INTERNAL_RENDER_SCALE: f32 = 0.25;
    pub const MAX_INTERNAL_RENDER_SCALE: f32 = 1.0;

    pub fn new(internal_render_scale: f32) -> Self {
        let mut profile = Self {
            internal_render_scale: Self::MAX_INTERNAL_RENDER_SCALE,
        };
        profile.set_internal_render_scale(internal_render_scale);
        profile
    }

    pub fn internal_render_scale(self) -> f32 {
        self.internal_render_scale
    }

    pub fn render_scale_percent(self) -> u8 {
        (self.internal_render_scale * 100.0).round() as u8
    }

    pub fn set_internal_render_scale(&mut self, internal_render_scale: f32) {
        self.internal_render_scale = internal_render_scale.clamp(
            Self::MIN_INTERNAL_RENDER_SCALE,
            Self::MAX_INTERNAL_RENDER_SCALE,
        );
    }
}

const QUALITY_PRESETS: &[f32] = &[1.0, 0.75, 0.5, 0.33, 0.25];
const QUALITY_WHEEL_STEP: f32 = 0.05;

/// Host ↔ panel bridge for the camera's render depth. The host owns the
/// camera, so the panel never edits it directly: wheel steps accumulate here
/// and the host drains them into `Camera::pan_focus_depth` once per frame,
/// then publishes the live focus depth back for the row's display.
pub struct CameraDepthLink {
    pending_steps: StdCell<i32>,
    current: StdCell<i32>,
}

impl CameraDepthLink {
    /// Host-side constructor: the depth starts centered at 0.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            pending_steps: StdCell::new(0),
            current: StdCell::new(0),
        })
    }

    /// Panel-side wheel input: one signed step per wheel notch, matching the
    /// canvas depth scroll's per-event step counting.
    pub fn scroll(&self, steps: i32) {
        self.pending_steps.set(self.pending_steps.get() + steps);
    }

    /// Host-side drain: apply the accumulated steps to the camera.
    pub fn drain_pending(&self) -> i32 {
        let steps = self.pending_steps.get();
        self.pending_steps.set(0);
        steps
    }

    /// Host-side publish: the live focus depth for the row's display.
    pub fn set_current(&self, depth: i32) {
        self.current.set(depth);
    }

    pub fn current(&self) -> i32 {
        self.current.get()
    }
}

/// Host ↔ panel bridge for the camera's rendered-layer count. Same shape as
/// [`CameraDepthLink`]: the host owns the camera, so wheel steps accumulate
/// here, the host drains them into `Camera::visible_plane_radius` (clamped
/// to `0..=MAX_VISIBLE_PLANE_RADIUS`) once per frame, then publishes the
/// live radius back for the row's display.
pub struct CameraLayersLink {
    pending_steps: StdCell<i32>,
    current: StdCell<i32>,
}

impl CameraLayersLink {
    /// Host-side constructor: the count starts at the camera default (8).
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            pending_steps: StdCell::new(0),
            current: StdCell::new(0),
        })
    }

    /// Panel-side wheel input: one signed step per wheel notch.
    pub fn scroll(&self, steps: i32) {
        self.pending_steps.set(self.pending_steps.get() + steps);
    }

    /// Host-side drain: apply the accumulated steps to the camera.
    pub fn drain_pending(&self) -> i32 {
        let steps = self.pending_steps.get();
        self.pending_steps.set(0);
        steps
    }

    /// Host-side publish: the live `visible_plane_radius` for the row's display.
    pub fn set_current(&self, radius: i32) {
        self.current.set(radius);
    }

    pub fn current(&self) -> i32 {
        self.current.get()
    }
}

/// One drained host command from [`CameraZoomLink`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraZoomCommand {
    None,
    /// Apply N multiplicative zoom steps — the exact − and + key semantics
    /// (`Camera::zoom_in`/`zoom_out`, which already clamp to MIN/MAX zoom).
    Steps(i32),
    /// Snap the zoom to an absolute preset value (clamp host-side to
    /// `Camera::MIN_ZOOM`..`MAX_ZOOM`).
    Set(f32),
}

/// Host ↔ panel bridge for the camera's zoom. Same shape as
/// [`CameraDepthLink`] plus one absolute-preset channel: the host owns the
/// camera, so wheel steps accumulate here, the host drains them into the
/// same `Camera::zoom_in`/`zoom_out` steps the − and + keys drive (or snaps
/// to a clicked preset), then publishes the live zoom back for the row's
/// display.
pub struct CameraZoomLink {
    pending_steps: StdCell<i32>,
    pending_target: StdCell<Option<f32>>,
    current: StdCell<f32>,
}

impl CameraZoomLink {
    /// Host-side constructor: the zoom starts at the camera default (1.0).
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            pending_steps: StdCell::new(0),
            pending_target: StdCell::new(None),
            current: StdCell::new(1.0),
        })
    }

    /// Panel-side wheel input: one signed step per wheel notch, wheel up =
    /// zoom in — matching the wheel binding that shares the − / + actions.
    pub fn scroll(&self, steps: i32) {
        self.pending_steps.set(self.pending_steps.get() + steps);
    }

    /// Panel-side click input: snap the zoom to an absolute preset value.
    pub fn request_zoom(&self, zoom: f32) {
        self.pending_target.set(Some(zoom));
    }

    /// Host-side drain: one command per frame, preset targets outranking
    /// accumulated steps (a click is a deliberate snap).
    pub fn drain_pending(&self) -> CameraZoomCommand {
        if let Some(target) = self.pending_target.replace(None) {
            return CameraZoomCommand::Set(target);
        }
        let steps = self.pending_steps.replace(0);
        if steps != 0 {
            CameraZoomCommand::Steps(steps)
        } else {
            CameraZoomCommand::None
        }
    }

    /// Host-side publish: the live camera zoom for the row's display.
    pub fn set_current(&self, zoom: f32) {
        self.current.set(zoom);
    }

    pub fn current(&self) -> f32 {
        self.current.get()
    }
}

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
    parallax: Rc<RefCell<ParallaxProfile>>,
    depth: Rc<CameraDepthLink>,
    layers: Rc<CameraLayersLink>,
    zoom: Rc<CameraZoomLink>,
    quality: Rc<RefCell<RenderQualityProfile>>,
    palette: UiPalette,
    gizmos: GizmoBar,
    gizmo_state: GizmoState,
    hidden: bool,
    /// The row currently under the pointer, driving the responsive
    /// value/label highlight.
    hovered_row: Option<usize>,
}

impl CameraPerspectiveModule {
    pub fn new(
        id: impl Into<String>,
        rect: ModuleRect,
        profile: Rc<RefCell<PerspectiveProfile>>,
        parallax: Rc<RefCell<ParallaxProfile>>,
        depth: Rc<CameraDepthLink>,
        layers: Rc<CameraLayersLink>,
        zoom: Rc<CameraZoomLink>,
    ) -> Self {
        Self {
            id: id.into(),
            rect,
            profile,
            parallax,
            depth,
            layers,
            zoom,
            quality: Rc::new(RefCell::new(RenderQualityProfile::default())),
            palette: UiPalette::default(),
            gizmos: GizmoBar::standard(),
            gizmo_state: GizmoState::new(),
            hidden: false,
            hovered_row: None,
        }
    }

    /// The perspective profile this module edits, for hosts to bind into
    /// their camera.
    pub fn profile(&self) -> Rc<RefCell<PerspectiveProfile>> {
        self.profile.clone()
    }

    /// The parallax profile this module edits, for hosts to bind into their
    /// camera (the host still feeds the pointer offset every frame).
    pub fn parallax_profile(&self) -> Rc<RefCell<ParallaxProfile>> {
        self.parallax.clone()
    }

    /// The depth link this module scrolls, for hosts to drain into their
    /// camera's focus depth each frame.
    pub fn depth_link(&self) -> Rc<CameraDepthLink> {
        self.depth.clone()
    }

    /// The layers link this module scrolls, for hosts to drain into their
    /// camera's `visible_plane_radius` each frame.
    pub fn layers_link(&self) -> Rc<CameraLayersLink> {
        self.layers.clone()
    }

    /// The zoom link this module scrolls, for hosts to drain into the same
    /// camera zoom the − and + keys drive each frame.
    pub fn zoom_link(&self) -> Rc<CameraZoomLink> {
        self.zoom.clone()
    }

    /// Binds a host-owned presentation-quality preference into this panel.
    /// The host applies its current scale to the presentation surface.
    pub fn with_render_quality(mut self, quality: Rc<RefCell<RenderQualityProfile>>) -> Self {
        self.quality = quality;
        self
    }

    /// Binds the host's shared responsive `UiPalette` (J 2026-09-12): the
    /// panel previously kept its own default palette and read as the one
    /// off-palette module on screen. `UiPalette` clones share their color
    /// slots through an `Rc`, so this keeps UI COLORS live for the panel.
    pub fn with_palette(mut self, palette: UiPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn render_quality_profile(&self) -> Rc<RefCell<RenderQualityProfile>> {
        self.quality.clone()
    }

    fn knob_row_y(&self, index: usize) -> i32 {
        PropertyRows::top_row_y(self.rect) - index as i32
    }

    /// Which knob/parallax row (if any) is under this screen-space point.
    /// Rows stack downward from the top content row, one line each, matching
    /// the property-rows convention (draw output is module-local, events are
    /// screen-space like `PropertyRows::hit_test`).
    fn row_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.rect.contains(x, y) {
            return None;
        }
        let local_y = y - self.rect.y0;
        (0..ROW_COUNT).find(|&index| self.knob_row_y(index) == local_y)
    }

    fn apply_wheel(
        &self,
        profile: &mut PerspectiveProfile,
        parallax: &mut ParallaxProfile,
        index: usize,
        delta_y: f32,
    ) {
        if index == PARALLAX_TOGGLE_ROW {
            // Wheel over the toggle flips it, matching the click behavior.
            parallax.enabled = !parallax.enabled;
            Self::ensure_usable_strength(parallax);
            return;
        }
        if index == DEPTH_ROW {
            // One signed step per wheel notch, matching the canvas depth
            // scroll: wheel up steps toward the viewer, wheel down away.
            let steps = if delta_y > 0.0 {
                1
            } else if delta_y < 0.0 {
                -1
            } else {
                0
            };
            self.depth.scroll(steps);
            return;
        }
        if index == LAYERS_ROW {
            // One rendered layer per wheel notch, clamped host-side.
            let steps = if delta_y > 0.0 {
                1
            } else if delta_y < 0.0 {
                -1
            } else {
                0
            };
            self.layers.scroll(steps);
            return;
        }
        if index == ZOOM_ROW {
            // One multiplicative zoom step per wheel notch, wheel up = zoom
            // in — the same step the − and + keys drive.
            let steps = if delta_y > 0.0 {
                1
            } else if delta_y < 0.0 {
                -1
            } else {
                0
            };
            self.zoom.scroll(steps);
            return;
        }
        if index == QUALITY_ROW {
            let step = if delta_y > 0.0 {
                QUALITY_WHEEL_STEP
            } else if delta_y < 0.0 {
                -QUALITY_WHEEL_STEP
            } else {
                return;
            };
            let mut quality = self.quality.borrow_mut();
            let next = quality.internal_render_scale() + step;
            quality.set_internal_render_scale(next);
            return;
        }
        let knob = if index == PARALLAX_STRENGTH_ROW {
            &PARALLAX_STRENGTH_KNOB
        } else {
            knob(index)
        };
        let step = if delta_y > 0.0 {
            knob.wheel_step
        } else if delta_y < 0.0 {
            -knob.wheel_step
        } else {
            return;
        };
        let value = if index == PARALLAX_STRENGTH_ROW {
            parallax.strength + step
        } else {
            knob_value(profile, index) + step
        };
        let value = (value).clamp(knob.min, knob.max);
        let rounded = (value * 100.0).round() / 100.0;
        if index == PARALLAX_STRENGTH_ROW {
            parallax.strength = rounded;
        } else {
            set_knob_value(profile, index, rounded);
        }
    }

    fn cycle_preset(
        &self,
        profile: &mut PerspectiveProfile,
        parallax: &mut ParallaxProfile,
        index: usize,
    ) {
        if index == PARALLAX_TOGGLE_ROW {
            parallax.enabled = !parallax.enabled;
            Self::ensure_usable_strength(parallax);
            return;
        }
        if index == DEPTH_ROW {
            // Click re-centers the render depth at 0.
            self.depth.scroll(-self.depth.current());
            return;
        }
        if index == LAYERS_ROW {
            // Click cycles the rendered-layer presets.
            let current = self.layers.current();
            let position = LAYERS_PRESETS
                .iter()
                .enumerate()
                .min_by(|a, b| (*a.1 - current).abs().cmp(&(*b.1 - current).abs()))
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next = LAYERS_PRESETS[(position + 1) % LAYERS_PRESETS.len()];
            self.layers.scroll(next - current);
            return;
        }
        if index == ZOOM_ROW {
            // Click cycles upward through the zoom presets and wraps, the
            // quality row's click-preset behavior; the host clamps the snap
            // to the camera's zoom range.
            let current = self.zoom.current();
            let next = ZOOM_PRESETS
                .iter()
                .copied()
                .find(|preset| *preset > current + f32::EPSILON)
                .unwrap_or(*ZOOM_PRESETS.first().expect("zoom presets are non-empty"));
            self.zoom.request_zoom(next);
            return;
        }
        if index == QUALITY_ROW {
            // Click cycles upward through the presets and wraps, matching
            // the sibling knob rows' click-cycles-presets behavior.
            let mut quality = self.quality.borrow_mut();
            let current = quality.internal_render_scale();
            let next = QUALITY_PRESETS
                .iter()
                .rev()
                .copied()
                .find(|preset| *preset > current + f32::EPSILON)
                .unwrap_or(*QUALITY_PRESETS.last().expect("quality presets are non-empty"));
            quality.set_internal_render_scale(next);
            return;
        }
        let knob = if index == PARALLAX_STRENGTH_ROW {
            &PARALLAX_STRENGTH_KNOB
        } else {
            knob(index)
        };
        let current = if index == PARALLAX_STRENGTH_ROW {
            parallax.strength
        } else {
            knob_value(profile, index)
        };
        let position = knob
            .presets
            .iter()
            .enumerate()
            .min_by(|a, b| (a.1 - current).abs().total_cmp(&(b.1 - current).abs()))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let next = knob.presets[(position + 1) % knob.presets.len()];
        if index == PARALLAX_STRENGTH_ROW {
            parallax.strength = next;
        } else {
            set_knob_value(profile, index, next);
        }
    }

    /// Enabling parallax through the panel guarantees a non-zero strength so
    /// the first toggle is immediately visible.
    fn ensure_usable_strength(parallax: &mut ParallaxProfile) {
        if parallax.enabled && parallax.strength <= 0.0 {
            parallax.strength = ParallaxProfile::DEFAULT_STRENGTH;
        }
    }
}

impl Module for CameraPerspectiveModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn rect(&self) -> ModuleRect {
        self.rect
    }

    /// Tooltip hotspots: the module's gizmo bar plus a title hotspot naming
    /// the panel and its camera truth.
    fn hotspots(&self) -> Vec<Hotspot> {
        let custom = vec![title_hotspot(
            self.rect,
            self.gizmos.title_start_x(),
            "perspective module",
            "adjusts the camera settings per artfile. parallax triggers with mouse movement, scale and position interact with relative layer depth. turn it all down to 0 for orthographic!",
        )];
        self.gizmos.hotspots_with(self.rect, custom)
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
            cells.extend(
                self.gizmos
                    .cells(self.rect, &self.gizmo_state, &self.palette),
            );
        }

        let (content_x, content_y) = PanelChrome::content_origin();
        let (_, content_height) = PanelChrome::content_size(self.rect);
        let top_row_y = content_y + (content_height - 1).max(0);
        let value_x = content_x + 9;
        let profile = self.profile.borrow();
        let parallax = self.parallax.borrow();
        let quality = self.quality.borrow();

        for index in 0..ROW_COUNT {
            let y = top_row_y - index as i32;
            let (label, value, engaged) = if index == PARALLAX_TOGGLE_ROW {
                (
                    "parallax",
                    if parallax.enabled { "on" } else { "off" }.to_string(),
                    parallax.enabled,
                )
            } else if index == PARALLAX_STRENGTH_ROW {
                ("str", format!("{:.2}", parallax.strength), parallax.enabled)
            } else if index == DEPTH_ROW {
                ("depth", format!("{}", self.depth.current()), true)
            } else if index == LAYERS_ROW {
                ("layers", format!("{}", self.layers.current()), true)
            } else if index == ZOOM_ROW {
                ("zoom", format!("{:.2}", self.zoom.current()), true)
            } else if index == QUALITY_ROW {
                ("quality", format!("{}%", quality.render_scale_percent()), true)
            } else {
                (
                    knob(index).label,
                    format!("{:.2}", knob_value(&profile, index)),
                    true,
                )
            };
            // Standard responsive role ladder (matching the controls panel):
            // labels Medium, values Bright at rest and Vivid when the row is
            // engaged (parallax on) or under the pointer.
            let hovered = self.hovered_row == Some(index);
            for (offset, glyph) in label.chars().enumerate() {
                cells.push(Cell {
                    position: CellPoint {
                        x: content_x + offset as i32,
                        y,
                        z: 0,
                    },
                    graphic: CellGraphic::Glyph(glyph),
                    color: self.palette.get(if hovered {
                        UiColorRole::Vivid
                    } else {
                        UiColorRole::Medium
                    }),
                    weight: CellWeight::from_index_clamped(1),
                    ..Cell::default()
                });
            }
            for (offset, glyph) in value.chars().enumerate() {
                cells.push(Cell {
                    position: CellPoint {
                        x: value_x + offset as i32,
                        y,
                        z: 0,
                    },
                    graphic: CellGraphic::Glyph(glyph),
                    color: self.palette.get(if engaged || hovered {
                        UiColorRole::Vivid
                    } else {
                        UiColorRole::Bright
                    }),
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
                    if let Some(index) = self.row_at(x, y) {
                        let mut profile = self.profile.borrow_mut();
                        let mut parallax = self.parallax.borrow_mut();
                        self.cycle_preset(&mut profile, &mut parallax, index);
                    }
                }
            }
            ModulePointerEvent::Move { x, y } => {
                self.gizmo_state.note_pointer(&self.gizmos, self.rect, x, y);
                if let Some(next_rect) = self.gizmo_state.drag_rect(x, y) {
                    self.rect = next_rect;
                }
                self.hovered_row = self.row_at(x, y);
            }
            ModulePointerEvent::Up { .. } => self.gizmo_state.end_drag(),
            ModulePointerEvent::Enter => self.gizmo_state.set_hovered(true),
            ModulePointerEvent::Leave => {
                self.gizmo_state.set_hovered(false);
                self.hovered_row = None;
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

    fn on_wheel(&mut self, x: i32, y: i32, _delta_x: f32, delta_y: f32) -> bool {
        let Some(index) = self.row_at(x, y) else {
            // Off the panel rows: fall through to viewport camera behavior.
            return false;
        };
        let mut profile = self.profile.borrow_mut();
        let mut parallax = self.parallax.borrow_mut();
        self.apply_wheel(&mut profile, &mut parallax, index, delta_y);
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
            // 13 tall: content 10 = 9 rows (scale/position/floor/parallax/
            // str/depth/layers/zoom/quality) plus the reserved bottom hint row.
            y1: 23,
        }
    }

    type ModuleFixture = (
        CameraPerspectiveModule,
        Rc<RefCell<PerspectiveProfile>>,
        Rc<RefCell<ParallaxProfile>>,
        Rc<CameraDepthLink>,
        Rc<CameraLayersLink>,
        Rc<CameraZoomLink>,
    );

    fn module() -> ModuleFixture {
        make_module()
    }

    fn make_module() -> ModuleFixture {
        let profile = Rc::new(RefCell::new(PerspectiveProfile::default()));
        let parallax = Rc::new(RefCell::new(ParallaxProfile::default()));
        let depth = CameraDepthLink::new();
        let layers = CameraLayersLink::new();
        let zoom = CameraZoomLink::new();
        let module = CameraPerspectiveModule::new(
            "camera_perspective",
            rect(),
            profile.clone(),
            parallax.clone(),
            depth.clone(),
            layers.clone(),
            zoom.clone(),
        );
        (module, profile, parallax, depth, layers, zoom)
    }

    fn scale_row_y() -> i32 {
        // Screen-space: knob rows are module-local inside draw, events are
        // screen-space, so add the rect origin.
        rect().y0 + PropertyRows::top_row_y(rect())
    }

    fn quality_row_y() -> i32 {
        scale_row_y() - QUALITY_ROW as i32
    }

    fn gizmo_row_y() -> i32 {
        // Gizmo glyphs sit on the chrome's top border row: local y = height-1.
        rect().y0 + (rect().y1 - rect().y0) - 1
    }

    #[test]
    fn clicking_the_close_gizmo_hides_the_panel_and_recall_revives_it() {
        let (mut module, _profile, _parallax, _depth, _layers, _zoom) = module();
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
        let (mut module, _profile, _parallax, _depth, _layers, _zoom) = module();
        let before = module.rect();

        module.on_pointer_event(ModulePointerEvent::Click {
            x: rect().x0 + 1,
            y: gizmo_row_y(),
            button: ModulePointerButton::Left,
        });
        assert!(
            module.wants_pointer_capture(),
            "move drag must hold capture"
        );

        module.on_pointer_event(ModulePointerEvent::Move {
            x: rect().x0 + 4,
            y: gizmo_row_y() + 2,
        });
        assert_eq!(
            module.rect(),
            ModuleRect {
                x0: before.x0 + 3,
                y0: before.y0 + 2,
                x1: before.x1 + 3,
                y1: before.y1 + 2,
            }
        );

        module.on_pointer_event(ModulePointerEvent::Up { x: 0, y: 0 });
        assert!(!module.wants_pointer_capture(), "release must end the drag");
    }

    #[test]
    fn panel_ui_state_persists_rect_seamless_and_hidden() {
        let (mut module, _profile, _parallax, _depth, _layers, _zoom) = module();
        module.on_pointer_event(ModulePointerEvent::Click {
            x: rect().x0 + 7,
            y: gizmo_row_y(),
            button: ModulePointerButton::Left,
        });
        assert!(module.gizmo_state.is_seamless());

        let saved = module.persisted_ui_state().expect("ui state snapshot");
        let (mut revived, _profile, _parallax, _depth, _layers, _zoom) = make_module();
        revived.apply_persisted_ui_state(&saved);
        assert_eq!(revived.rect(), module.rect());
        assert!(revived.gizmo_state.is_seamless());
    }

    #[test]
    fn draw_output_is_anchored_at_the_module_rect_not_the_screen_origin() {
        // Regression: a group origin of (0,0) drew the panel at the screen's
        // bottom-left corner while hit-testing stayed at `self.rect`, so the
        // visible panel was dead to input.
        let (module, _profile, _parallax, _depth, _layers, _zoom) = module();
        let group = module.draw();
        let origin = group.origin;
        assert_eq!((origin.x, origin.y), (rect().x0, rect().y0));
    }

    #[test]
    fn wheel_over_a_knob_row_fine_tunes_that_knob() {
        let (mut module, profile, _parallax, _depth, _layers, _zoom) = module();
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
        let (mut module, profile, _parallax, _depth, _layers, _zoom) = module();
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
    fn quality_row_changes_the_bound_render_scale_and_clamps_to_user_range() {
        let (mut module, _profile, _parallax, _depth, _layers, _zoom) = module();
        let quality = Rc::new(RefCell::new(RenderQualityProfile::default()));
        module = module.with_render_quality(quality.clone());

        module.on_wheel(12, quality_row_y(), 0.0, -1.0);
        assert_eq!(quality.borrow().render_scale_percent(), 95);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y: quality_row_y(),
            button: ModulePointerButton::Left,
        });
        assert_eq!(quality.borrow().render_scale_percent(), 100);
        for _ in 0..30 {
            module.on_wheel(12, quality_row_y(), 0.0, -1.0);
        }
        assert_eq!(quality.borrow().render_scale_percent(), 25);
    }

    #[test]
    fn clicking_a_knob_row_cycles_presets_and_wraps() {
        let (mut module, profile, _parallax, _depth, _layers, _zoom) = module();
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
        let (mut module, _profile, _parallax, _depth, _layers, _zoom) = module();
        assert!(!module.on_wheel(12, rect().y0 + 1, 0.0, 1.0));
        // One line below the bottom (quality) row: still fall-through.
        assert!(!module.on_wheel(12, quality_row_y() - 1, 0.0, 1.0));
        assert!(!module.on_wheel(12, rect().y1 - 1, 0.0, 1.0));
    }

    #[test]
    fn edits_land_in_the_shared_profile_the_host_binds() {
        let (mut module, profile, _parallax, _depth, _layers, _zoom) = module();
        let floor_y = scale_row_y() - 2;
        module.on_wheel(12, floor_y, 0.0, -1.0);
        assert!((profile.borrow().near_floor_fraction - 0.26).abs() < 1e-4);
        assert_eq!(module.profile(), profile);
    }

    #[test]
    fn draw_renders_knob_labels_and_live_values() {
        let (module, profile, _parallax, _depth, _layers, _zoom) = module();
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
            assert!(
                glyphs.iter().any(|(position, glyph)| {
                    position.y == label_row_y - row_offset as i32 && label.starts_with(*glyph)
                }),
                "missing {label}"
            );
        }
        // The live value renders, including the edited one.
        for expected in ['1', '.', '2', '5'] {
            assert!(glyphs
                .iter()
                .any(|(position, glyph)| { position.y == label_row_y && glyph == &expected }));
        }
    }

    fn row_screen_y(index: usize) -> i32 {
        rect().y0 + PropertyRows::top_row_y(rect()) - index as i32
    }

    #[test]
    fn clicking_the_parallax_row_toggles_it_and_seeds_a_usable_strength() {
        let (mut module, _profile, parallax, _depth, _layers, _zoom) = module();
        let y = row_screen_y(PARALLAX_TOGGLE_ROW);
        let x = rect().x0 + 3;

        module.on_pointer_event(ModulePointerEvent::Click {
            x,
            y,
            button: ModulePointerButton::Left,
        });
        let state = parallax.borrow();
        assert!(state.enabled, "click must enable parallax");
        assert!(state.strength > 0.0, "enabling must seed a usable strength");
        drop(state);

        module.on_pointer_event(ModulePointerEvent::Click {
            x,
            y,
            button: ModulePointerButton::Left,
        });
        assert!(
            !parallax.borrow().enabled,
            "second click must disable parallax"
        );
    }

    #[test]
    fn wheel_over_the_strength_row_fine_tunes_and_clamps() {
        let (mut module, _profile, parallax, _depth, _layers, _zoom) = module();
        let y = row_screen_y(PARALLAX_STRENGTH_ROW);

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        let strength = parallax.borrow().strength;
        assert!(
            (strength - 0.16).abs() < 1e-4,
            "default 0.15 + 0.01, got {strength}"
        );

        for _ in 0..80 {
            module.on_wheel(12, y, 0.0, 1.0);
        }
        assert!((parallax.borrow().strength - ParallaxProfile::MAX_STRENGTH).abs() < 1e-4);
    }

    #[test]
    fn clicking_the_strength_row_cycles_presets() {
        let (mut module, _profile, parallax, _depth, _layers, _zoom) = module();
        let y = row_screen_y(PARALLAX_STRENGTH_ROW);

        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        // Default 0.15 is a preset: the next click lands on 0.25.
        assert!((parallax.borrow().strength - 0.25).abs() < 1e-4);
    }

    #[test]
    fn wheel_on_the_toggle_row_flips_parallax_without_touching_perspective() {
        let (mut module, profile, parallax, _depth, _layers, _zoom) = module();
        let y = row_screen_y(PARALLAX_TOGGLE_ROW);

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(parallax.borrow().enabled);
        assert!((profile.borrow().scale_strength - 0.9).abs() < 1e-4);
    }

    #[test]
    fn draw_shows_the_parallax_state_and_strength() {
        let (module, _profile, parallax, _depth, _layers, _zoom) = module();
        parallax.borrow_mut().enabled = true;
        parallax.borrow_mut().strength = 0.2;
        let group = module.draw();
        let label_row_y = PropertyRows::top_row_y(rect());
        let glyphs: Vec<(CellPoint, char)> = group
            .iter_cells()
            .filter_map(|cell| match &cell.graphic {
                CellGraphic::Glyph(glyph) => Some((cell.position, *glyph)),
                _ => None,
            })
            .collect();

        let row_text = |row_y: i32| -> String {
            let mut chars: Vec<(i32, char)> = glyphs
                .iter()
                .filter(|(position, _)| position.y == row_y)
                .map(|(position, glyph)| (position.x, *glyph))
                .collect();
            chars.sort_by_key(|(x, _)| *x);
            chars.into_iter().map(|(_, glyph)| glyph).collect()
        };
        let toggle_row = row_text(label_row_y - PARALLAX_TOGGLE_ROW as i32);
        assert!(toggle_row.contains("parallax"), "got {toggle_row:?}");
        assert!(toggle_row.contains("on"), "got {toggle_row:?}");
        let strength_row = row_text(label_row_y - PARALLAX_STRENGTH_ROW as i32);
        assert!(strength_row.contains("0.20"), "got {strength_row:?}");
    }

    #[test]
    fn panel_edits_land_in_the_shared_parallax_profile_the_host_binds() {
        let (module, _profile, parallax, _depth, _layers, _zoom) = module();
        assert_eq!(module.parallax_profile(), parallax);
    }

    fn depth_row_screen_y() -> i32 {
        row_screen_y(DEPTH_ROW)
    }

    fn layers_row_screen_y() -> i32 {
        row_screen_y(LAYERS_ROW)
    }

    fn zoom_row_screen_y() -> i32 {
        row_screen_y(ZOOM_ROW)
    }

    #[test]
    fn wheel_over_the_layers_row_accumulates_host_drained_steps() {
        let (mut module, _profile, _parallax, _depth, layers, _zoom) = module();
        let y = layers_row_screen_y();

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, -1.0));
        assert_eq!(layers.drain_pending(), 1, "two up, one down = +1 step");
        assert_eq!(layers.drain_pending(), 0, "drain must empty the queue");
    }

    #[test]
    fn clicking_the_layers_row_cycles_presets_through_the_link() {
        let (mut module, _profile, _parallax, _depth, layers, _zoom) = module();
        let y = layers_row_screen_y();

        // Host publishes the live radius (default 8 is a preset): the click
        // must queue the step delta onto the next preset.
        layers.set_current(8);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(layers.drain_pending(), 4, "8 -> 12");

        // And from the top preset it wraps to the smallest one.
        layers.set_current(MAX_VISIBLE_PLANE_RADIUS);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(
            layers.drain_pending(),
            -MAX_VISIBLE_PLANE_RADIUS,
            "512 -> 0"
        );
    }

    #[test]
    fn draw_shows_the_host_published_layer_count() {
        let (module, _profile, _parallax, _depth, layers, _zoom) = module();
        layers.set_current(12);
        let group = module.draw();
        let label_row_y = PropertyRows::top_row_y(rect());
        let row_text = row_text_at(&group, label_row_y - LAYERS_ROW as i32);
        assert!(row_text.contains("layers"), "got {row_text:?}");
        assert!(row_text.contains("12"), "got {row_text:?}");
    }

    #[test]
    fn wheel_over_the_zoom_row_accumulates_host_drained_steps() {
        // J 2026-09-12: the zoom row drives the same camera zoom the − and +
        // keys drive — one multiplicative step per wheel notch.
        let (mut module, _profile, _parallax, _depth, _layers, zoom) = module();
        let y = zoom_row_screen_y();

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, -1.0));
        assert_eq!(
            zoom.drain_pending(),
            CameraZoomCommand::Steps(1),
            "two up, one down = one zoom-in step"
        );
        assert_eq!(zoom.drain_pending(), CameraZoomCommand::None);
    }

    #[test]
    fn clicking_the_zoom_row_snaps_to_the_next_preset_through_the_link() {
        let (mut module, _profile, _parallax, _depth, _layers, zoom) = module();
        let y = zoom_row_screen_y();

        zoom.set_current(1.0);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(zoom.drain_pending(), CameraZoomCommand::Set(2.0), "1.0 -> 2.0");

        // From the top preset it wraps to the smallest one.
        zoom.set_current(4.0);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(
            zoom.drain_pending(),
            CameraZoomCommand::Set(0.05),
            "4.0 -> 0.05"
        );

        // A preset snap outranks any accumulated wheel steps for the frame.
        assert!(module.on_wheel(12, y, 0.0, 1.0));
        zoom.set_current(0.15);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(zoom.drain_pending(), CameraZoomCommand::Set(0.3));
        assert_eq!(
            zoom.drain_pending(),
            CameraZoomCommand::Steps(1),
            "the earlier queued wheel step still drains after the snap"
        );
        assert_eq!(zoom.drain_pending(), CameraZoomCommand::None);
    }

    #[test]
    fn draw_shows_the_host_published_zoom_between_layers_and_quality() {
        let (module, _profile, _parallax, _depth, _layers, zoom) = module();
        zoom.set_current(0.15);
        let group = module.draw();
        let label_row_y = PropertyRows::top_row_y(rect());
        let zoom_text = row_text_at(&group, label_row_y - ZOOM_ROW as i32);
        assert!(zoom_text.contains("zoom"), "got {zoom_text:?}");
        assert!(zoom_text.contains("0.15"), "got {zoom_text:?}");
        let layers_text = row_text_at(&group, label_row_y - LAYERS_ROW as i32);
        let quality_text = row_text_at(&group, label_row_y - QUALITY_ROW as i32);
        assert!(
            layers_text.contains("layers") && quality_text.contains("quality"),
            "zoom must sit between the layers and quality rows"
        );
    }

    #[test]
    fn wheel_over_the_depth_row_accumulates_host_drained_steps() {
        let (mut module, profile, parallax, depth, _layers, _zoom) = module();
        let y = depth_row_screen_y();

        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, 1.0));
        assert!(module.on_wheel(12, y, 0.0, -1.0));
        assert_eq!(depth.drain_pending(), 1, "two up, one down = +1 step");
        assert_eq!(depth.drain_pending(), 0, "drain must empty the queue");

        // The perspective/parallax rows are untouched by depth wheels.
        assert!((profile.borrow().scale_strength - 0.9).abs() < 1e-4);
        assert!(!parallax.borrow().enabled);
    }

    #[test]
    fn clicking_the_depth_row_asks_for_a_recenter_through_the_link() {
        let (mut module, _profile, _parallax, depth, _layers, _zoom) = module();
        let y = depth_row_screen_y();

        // Host publishes a live depth, then the click must undo it.
        depth.set_current(6);
        module.on_pointer_event(ModulePointerEvent::Click {
            x: 12,
            y,
            button: ModulePointerButton::Left,
        });
        assert_eq!(depth.drain_pending(), -6);
    }

    #[test]
    fn draw_shows_the_host_published_depth() {
        let (module, _profile, _parallax, depth, _layers, _zoom) = module();
        depth.set_current(-3);
        let group = module.draw();
        let label_row_y = PropertyRows::top_row_y(rect());
        let row_text = row_text_at(&group, label_row_y - DEPTH_ROW as i32);
        assert!(row_text.contains("depth"), "got {row_text:?}");
        assert!(row_text.contains("-3"), "got {row_text:?}");
    }

    fn row_text_at(group: &CellGroup, row_y: i32) -> String {
        let mut chars: Vec<(i32, char)> = group
            .iter_cells()
            .filter(|cell| cell.position.y == row_y)
            .filter_map(|cell| match &cell.graphic {
                CellGraphic::Glyph(glyph) => Some((cell.position.x, *glyph)),
                _ => None,
            })
            .collect();
        chars.sort_by_key(|(x, _)| *x);
        chars.into_iter().map(|(_, glyph)| glyph).collect()
    }
}
