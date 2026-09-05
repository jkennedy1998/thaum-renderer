use serde::{Deserialize, Serialize};

use crate::{
    Camera, CameraProjectionMode, CameraRoll, CameraSwing, CellPoint, ModuleRect,
    PerspectiveProfile, UiColorRole,
    UiPalette, WorldPoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedModuleRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl From<ModuleRect> for PersistedModuleRect {
    fn from(value: ModuleRect) -> Self {
        Self {
            x0: value.x0,
            y0: value.y0,
            x1: value.x1,
            y1: value.y1,
        }
    }
}

impl PersistedModuleRect {
    pub fn to_runtime(self) -> ModuleRect {
        ModuleRect {
            x0: self.x0,
            y0: self.y0,
            x1: self.x1,
            y1: self.y1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedModuleUiState {
    pub module_id: String,
    pub rect: PersistedModuleRect,
    pub is_seamless: bool,
    #[serde(default)]
    pub is_hidden: bool,
}

impl PersistedModuleUiState {
    pub fn new(
        module_id: impl Into<String>,
        rect: ModuleRect,
        is_seamless: bool,
        is_hidden: bool,
    ) -> Self {
        Self {
            module_id: module_id.into(),
            rect: rect.into(),
            is_seamless,
            is_hidden,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedCameraUiState {
    pub position: [i32; 3],
    pub focus_target: [i32; 3],
    pub swing: String,
    pub roll: String,
    pub projection_mode: String,
    pub visible_plane_radius: i32,
    pub visible_plane_depth_offset: i32,
    pub zoom: f32,
    pub hud_pan_offset: [i32; 2],
    #[serde(default = "default_perspective_scale_strength")]
    pub perspective_scale_strength: f32,
    #[serde(default = "default_perspective_position_strength")]
    pub perspective_position_strength: f32,
    #[serde(default = "default_perspective_near_floor_fraction")]
    pub perspective_near_floor_fraction: f32,
}

fn default_perspective_scale_strength() -> f32 {
    PerspectiveProfile::default().scale_strength
}

fn default_perspective_position_strength() -> f32 {
    PerspectiveProfile::default().position_strength
}

fn default_perspective_near_floor_fraction() -> f32 {
    PerspectiveProfile::default().near_floor_fraction
}

impl PersistedCameraUiState {
    pub fn from_runtime(camera: Camera) -> Self {
        Self {
            position: [camera.position.x, camera.position.y, camera.position.z],
            focus_target: [
                camera.focus_target.x,
                camera.focus_target.y,
                camera.focus_target.z,
            ],
            swing: format!("{:?}", camera.swing),
            roll: format!("{:?}", camera.roll),
            projection_mode: format!("{:?}", camera.projection_mode),
            visible_plane_radius: camera.visible_plane_radius,
            visible_plane_depth_offset: camera.visible_plane_depth_offset,
            zoom: camera.zoom,
            hud_pan_offset: [camera.hud_pan_offset.x, camera.hud_pan_offset.y],
            perspective_scale_strength: camera.perspective.scale_strength,
            perspective_position_strength: camera.perspective.position_strength,
            perspective_near_floor_fraction: camera.perspective.near_floor_fraction,
        }
    }

    pub fn apply_to_runtime(&self, camera: &mut Camera) {
        camera.position = WorldPoint {
            x: self.position[0],
            y: self.position[1],
            z: self.position[2],
        };
        camera.focus_target = WorldPoint {
            x: self.focus_target[0],
            y: self.focus_target[1],
            z: self.focus_target[2],
        };
        if let Some(swing) = swing_from_name(&self.swing) {
            camera.swing = swing;
        }
        if let Some(roll) = roll_from_name(&self.roll) {
            camera.roll = roll;
        }
        if let Some(projection_mode) = projection_mode_from_name(&self.projection_mode) {
            camera.projection_mode = projection_mode;
        }
        camera.visible_plane_radius = self.visible_plane_radius;
        camera.visible_plane_depth_offset = self.visible_plane_depth_offset;
        camera.zoom = self.zoom.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);
        camera.hud_pan_offset = CellPoint {
            x: self.hud_pan_offset[0],
            y: self.hud_pan_offset[1],
            z: 0,
        };
        camera.perspective = PerspectiveProfile {
            scale_strength: self.perspective_scale_strength,
            position_strength: self.perspective_position_strength,
            near_floor_fraction: self.perspective_near_floor_fraction,
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedUiPaletteColor {
    pub role: String,
    pub rgb: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedUiPaletteState {
    pub colors: Vec<PersistedUiPaletteColor>,
}

impl Default for PersistedUiPaletteState {
    fn default() -> Self {
        Self {
            colors: UiColorRole::ALL
                .iter()
                .map(|role| PersistedUiPaletteColor {
                    role: format!("{:?}", role),
                    rgb: UiPalette::default().get_rgb(*role),
                })
                .collect(),
        }
    }
}

impl PersistedUiPaletteState {
    pub fn from_runtime(palette: &UiPalette) -> Self {
        Self {
            colors: UiColorRole::ALL
                .iter()
                .map(|role| PersistedUiPaletteColor {
                    role: format!("{:?}", role),
                    rgb: palette.get_rgb(*role),
                })
                .collect(),
        }
    }

    pub fn apply_to_runtime(&self, palette: &UiPalette) {
        for color in &self.colors {
            if let Some(role) = ui_role_from_name(&color.role) {
                palette.set_rgb(role, color.rgb);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedRendererUiSessionState {
    pub camera: PersistedCameraUiState,
    pub modules: Vec<PersistedModuleUiState>,
    #[serde(default)]
    pub palette: PersistedUiPaletteState,
}

impl PersistedRendererUiSessionState {
    pub fn new(camera: Camera, modules: Vec<PersistedModuleUiState>, palette: &UiPalette) -> Self {
        Self {
            camera: PersistedCameraUiState::from_runtime(camera),
            modules,
            palette: PersistedUiPaletteState::from_runtime(palette),
        }
    }
}

fn swing_from_name(name: &str) -> Option<CameraSwing> {
    match name {
        "PosX" => Some(CameraSwing::PosX),
        "NegX" => Some(CameraSwing::NegX),
        "PosY" => Some(CameraSwing::PosY),
        "NegY" => Some(CameraSwing::NegY),
        "PosZ" => Some(CameraSwing::PosZ),
        "NegZ" => Some(CameraSwing::NegZ),
        _ => None,
    }
}

fn roll_from_name(name: &str) -> Option<CameraRoll> {
    match name {
        "Deg0" => Some(CameraRoll::Deg0),
        "Deg90" => Some(CameraRoll::Deg90),
        "Deg180" => Some(CameraRoll::Deg180),
        "Deg270" => Some(CameraRoll::Deg270),
        _ => None,
    }
}

fn projection_mode_from_name(name: &str) -> Option<CameraProjectionMode> {
    match name {
        "Perspective" => Some(CameraProjectionMode::Perspective),
        "Orthographic" => Some(CameraProjectionMode::Orthographic),
        _ => None,
    }
}

fn ui_role_from_name(name: &str) -> Option<UiColorRole> {
    match name {
        "Background" => Some(UiColorRole::Background),
        "Dimmest" => Some(UiColorRole::Dimmest),
        "Medium" => Some(UiColorRole::Medium),
        "Bright" => Some(UiColorRole::Bright),
        "Vivid" => Some(UiColorRole::Vivid),
        "LeftHand" => Some(UiColorRole::LeftHand),
        "RightHand" => Some(UiColorRole::RightHand),
        _ => None,
    }
}
