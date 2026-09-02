#[path = "projection/projection.rs"]
pub mod projection;
#[path = "roll/roll.rs"]
pub mod roll;
#[path = "screen-world-remap/screen_world_remap.rs"]
pub mod screen_world_remap;
#[path = "swing/swing.rs"]
pub mod swing;
#[path = "view-orientation/view_orientation.rs"]
pub mod view_orientation;

use crate::{coordinate_space::CellPoint, coordinate_space::WorldPoint, GlobalDirection};
pub use projection::{
    build_visible_plane_stack_around_focus, derive_visible_plane_stack_from_world_points,
    focus_plane_for_camera, project_flat_2d_world_to_view_plane,
    project_rotating_3d_world_to_view_plane, project_world_to_view_plane,
    project_world_to_view_plane_for_intake, projected_plane_is_visible,
    projected_plane_scale_factor, unproject_flat_2d_view_plane_to_local,
    unproject_view_plane_to_world, visible_plane_stack_for_camera, CameraProjectedPoint,
    CameraProjectionMode, VisiblePlaneStack,
};
pub use roll::CameraRoll;
pub use screen_world_remap::{
    remap_camera_units_to_active_plane_world, remap_camera_units_to_world_on_plane,
    remap_surface_units_to_active_plane_world, remap_surface_units_to_flat_2d_local,
};
pub use swing::CameraSwing;
pub use view_orientation::{
    active_depth_axis_for_swing, active_depth_direction_for_swing,
    camera_view_orientation_for_camera, camera_view_orientation_for_swing,
    project_world_relative_to_view, unproject_view_relative_to_world, CameraViewOrientation,
    ViewRelativePoint,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: WorldPoint,
    pub focus_target: WorldPoint,
    pub swing: CameraSwing,
    pub roll: CameraRoll,
    pub projection_mode: CameraProjectionMode,
    pub visible_plane_radius: i32,
    pub visible_plane_depth_offset: i32,
    pub zoom: f32,
    /// Screen-space pan applied to every `Flat2d` (HUD) cell, on top of its
    /// own local offset. Independent of `focus_target`/`swing`/`roll` so the
    /// 2D layer can be navigated (e.g. to reach off-screen panels) without
    /// disturbing the 3D scene pan.
    pub hud_pan_offset: CellPoint,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: WorldPoint::origin(),
            focus_target: WorldPoint::origin(),
            swing: CameraSwing::default(),
            roll: CameraRoll::default(),
            projection_mode: CameraProjectionMode::Perspective,
            visible_plane_radius: 8,
            visible_plane_depth_offset: 0,
            zoom: 1.0,
            hud_pan_offset: CellPoint::origin(),
        }
    }
}

impl Camera {
    pub const MIN_ZOOM: f32 = 0.05;
    pub const MAX_ZOOM: f32 = 4.0;
    pub const ZOOM_STEP_FACTOR: f32 = 1.25;

    pub fn turn_clockwise(&mut self) {
        self.swing_right();
    }

    pub fn turn_counter_clockwise(&mut self) {
        self.swing_left();
    }

    pub fn swing_left(&mut self) {
        self.apply_swing_transition(SwingDirection::Left);
    }

    pub fn swing_right(&mut self) {
        self.apply_swing_transition(SwingDirection::Right);
    }

    pub fn swing_up(&mut self) {
        self.apply_swing_transition(SwingDirection::Up);
    }

    pub fn swing_down(&mut self) {
        self.apply_swing_transition(SwingDirection::Down);
    }

    fn apply_swing_transition(&mut self, direction: SwingDirection) {
        let orientation = camera_view_orientation_for_camera(self.swing, self.roll);
        let target = match direction {
            SwingDirection::Left => CameraViewOrientation {
                depth: opposite_direction(orientation.right),
                right: orientation.depth,
                up: orientation.up,
            },
            SwingDirection::Right => CameraViewOrientation {
                depth: orientation.right,
                right: opposite_direction(orientation.depth),
                up: orientation.up,
            },
            SwingDirection::Up => CameraViewOrientation {
                depth: orientation.up,
                right: orientation.right,
                up: opposite_direction(orientation.depth),
            },
            SwingDirection::Down => CameraViewOrientation {
                depth: opposite_direction(orientation.up),
                right: orientation.right,
                up: orientation.depth,
            },
        };

        let (swing, roll) = decompose_orientation_to_camera(target);
        self.swing = swing;
        self.roll = roll;
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * Self::ZOOM_STEP_FACTOR).min(Self::MAX_ZOOM);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / Self::ZOOM_STEP_FACTOR).max(Self::MIN_ZOOM);
    }

    /// Pans `focus_target` sideways in the current view orientation — moves
    /// the 3D scene (Rotating3d content) under a screen-fixed HUD.
    pub fn pan_focus_right(&mut self, delta: i32) {
        self.step_focus_target(
            camera_view_orientation_for_camera(self.swing, self.roll).right,
            delta,
        );
    }

    /// Pans `focus_target` vertically in the current view orientation.
    pub fn pan_focus_up(&mut self, delta: i32) {
        self.step_focus_target(
            camera_view_orientation_for_camera(self.swing, self.roll).up,
            delta,
        );
    }

    /// Pans `focus_target` along the active depth axis (into/out of the screen).
    pub fn pan_focus_depth(&mut self, delta: i32) {
        self.step_focus_target(active_depth_direction_for_swing(self.swing), delta);
    }

    fn step_focus_target(&mut self, direction: GlobalDirection, delta: i32) {
        let unit = direction.unit_vector();
        self.focus_target.x += unit[0] * delta;
        self.focus_target.y += unit[1] * delta;
        self.focus_target.z += unit[2] * delta;
    }

    /// Pans the 2D HUD layer sideways, independent of `focus_target`/swing/roll.
    pub fn pan_hud_right(&mut self, delta: i32) {
        self.hud_pan_offset.x += delta;
    }

    /// Pans the 2D HUD layer vertically, independent of `focus_target`/swing/roll.
    pub fn pan_hud_up(&mut self, delta: i32) {
        self.hud_pan_offset.y += delta;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwingDirection {
    Left,
    Right,
    Up,
    Down,
}

const CAMERA_SWINGS: [CameraSwing; 6] = [
    CameraSwing::PosX,
    CameraSwing::NegX,
    CameraSwing::PosY,
    CameraSwing::NegY,
    CameraSwing::PosZ,
    CameraSwing::NegZ,
];

const CAMERA_ROLLS: [CameraRoll; 4] = [
    CameraRoll::Deg0,
    CameraRoll::Deg90,
    CameraRoll::Deg180,
    CameraRoll::Deg270,
];

fn opposite_direction(direction: GlobalDirection) -> GlobalDirection {
    match direction {
        GlobalDirection::East => GlobalDirection::West,
        GlobalDirection::West => GlobalDirection::East,
        GlobalDirection::Top => GlobalDirection::Bottom,
        GlobalDirection::Bottom => GlobalDirection::Top,
        GlobalDirection::South => GlobalDirection::North,
        GlobalDirection::North => GlobalDirection::South,
    }
}

fn decompose_orientation_to_camera(target: CameraViewOrientation) -> (CameraSwing, CameraRoll) {
    for swing in CAMERA_SWINGS {
        for roll in CAMERA_ROLLS {
            if camera_view_orientation_for_camera(swing, roll) == target {
                return (swing, roll);
            }
        }
    }

    unreachable!("every cube orientation should map to one swing+roll pair")
}

pub fn project_world_to_camera_units(camera: Camera, world: WorldPoint) -> [f32; 2] {
    let projected = project_world_to_view_plane(camera, world);
    [projected.u, projected.v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_world_to_camera_units_keeps_focus_target_centered() {
        let camera = Camera {
            position: WorldPoint { x: 9, y: 9, z: 9 },
            focus_target: WorldPoint { x: 3, y: -2, z: 4 },
            swing: CameraSwing::PosZ,
            ..Camera::default()
        };

        assert_eq!(
            project_world_to_camera_units(camera, camera.focus_target),
            [0.0, 0.0]
        );
    }

    #[test]
    fn turn_clockwise_swings_right_around_the_cube() {
        let mut camera = Camera::default();
        camera.turn_clockwise();

        assert_eq!(camera.swing, CameraSwing::PosX);
        assert_eq!(camera.roll, CameraRoll::Deg0);
    }

    #[test]
    fn repeated_swing_right_never_gets_stuck_on_vertical_faces() {
        let mut camera = Camera::default();

        camera.swing_up();
        assert_eq!(camera.swing, CameraSwing::PosY);
        assert_eq!(camera.roll, CameraRoll::Deg0);

        camera.swing_right();
        assert_eq!(camera.swing, CameraSwing::PosX);
        assert_eq!(camera.roll, CameraRoll::Deg270);

        camera.swing_right();
        assert_eq!(camera.swing, CameraSwing::NegY);
        assert_eq!(camera.roll, CameraRoll::Deg180);

        camera.swing_right();
        assert_eq!(camera.swing, CameraSwing::NegX);
        assert_eq!(camera.roll, CameraRoll::Deg90);
    }

    #[test]
    fn repeated_swing_up_walks_a_full_vertical_loop() {
        let mut camera = Camera::default();

        camera.swing_up();
        assert_eq!(
            (camera.swing, camera.roll),
            (CameraSwing::PosY, CameraRoll::Deg0)
        );
        camera.swing_up();
        assert_eq!(
            (camera.swing, camera.roll),
            (CameraSwing::NegZ, CameraRoll::Deg180)
        );
        camera.swing_up();
        assert_eq!(
            (camera.swing, camera.roll),
            (CameraSwing::NegY, CameraRoll::Deg0)
        );
        camera.swing_up();
        assert_eq!(
            (camera.swing, camera.roll),
            (CameraSwing::PosZ, CameraRoll::Deg0)
        );
    }

    #[test]
    fn swing_down_reverses_swing_up() {
        let mut camera = Camera {
            swing: CameraSwing::NegX,
            roll: CameraRoll::Deg90,
            ..Camera::default()
        };
        let start = camera;

        camera.swing_up();
        camera.swing_down();

        assert_eq!(camera.swing, start.swing);
        assert_eq!(camera.roll, start.roll);
    }

    #[test]
    fn pan_focus_right_and_up_move_focus_target_in_view_orientation() {
        let mut camera = Camera {
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        camera.pan_focus_right(2);
        camera.pan_focus_up(3);

        assert_ne!(camera.focus_target, WorldPoint::origin());
        assert_eq!(camera.hud_pan_offset, CellPoint::origin());
    }

    #[test]
    fn pan_focus_depth_moves_along_the_active_depth_axis() {
        let mut camera = Camera::default();
        camera.pan_focus_depth(4);

        assert_eq!(camera.focus_target, WorldPoint { x: 0, y: 0, z: 4 });
    }

    #[test]
    fn pan_hud_moves_only_the_hud_pan_offset() {
        let mut camera = Camera::default();
        camera.pan_hud_right(5);
        camera.pan_hud_up(-2);

        assert_eq!(camera.hud_pan_offset, CellPoint { x: 5, y: -2, z: 0 });
        assert_eq!(camera.focus_target, WorldPoint::origin());
    }

    #[test]
    fn roll_still_works_on_any_swung_side() {
        let mut camera = Camera::default();
        camera.swing_up();
        camera.swing_right();

        let before = camera_view_orientation_for_camera(camera.swing, camera.roll);
        camera.roll = camera.roll.rotate_clockwise();
        let after = camera_view_orientation_for_camera(camera.swing, camera.roll);

        assert_eq!(before.depth, after.depth);
        assert_ne!(before.right, after.right);
        assert_ne!(before.up, after.up);
    }
}
