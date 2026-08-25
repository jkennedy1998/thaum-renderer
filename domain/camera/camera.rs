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

use crate::{coordinate_space::WorldPoint, GlobalDirection};
pub use projection::{
    build_visible_plane_stack_around_focus, derive_visible_plane_stack_from_world_points,
    focus_plane_for_camera, project_flat_2d_world_to_view_plane,
    project_rotating_3d_world_to_view_plane, project_world_to_view_plane,
    project_world_to_view_plane_for_intake, projected_plane_is_visible,
    projected_plane_scale_factor, unproject_view_plane_to_world, visible_plane_stack_for_camera,
    CameraProjectedPoint, CameraProjectionMode, VisiblePlaneStack,
};
pub use roll::CameraRoll;
pub use screen_world_remap::{
    remap_camera_units_to_active_plane_world, remap_camera_units_to_world_on_plane,
    remap_surface_units_to_active_plane_world,
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
    fn project_world_to_camera_units_offsets_depth_up_and_right() {
        let camera = Camera::default();
        let projected = project_world_to_camera_units(camera, WorldPoint { x: 0, y: 0, z: 2 });

        assert!(projected[0] > 0.0);
        assert!(projected[1] < 0.0);
        assert!((projected[0] + projected[1]).abs() < 0.01);
    }

    #[test]
    fn project_world_to_camera_units_combines_world_xy_with_depth_offset() {
        let camera = Camera {
            position: WorldPoint::origin(),
            focus_target: WorldPoint { x: 1, y: 2, z: 3 },
            swing: CameraSwing::PosZ,
            ..Camera::default()
        };

        let projected = project_world_to_camera_units(camera, WorldPoint { x: 4, y: 7, z: 5 });

        assert!(projected[0] > 3.2 && projected[0] < 3.5);
        assert!(projected[1] > 4.2 && projected[1] < 4.5);
    }

    #[test]
    fn project_world_to_camera_units_rotates_depth_with_swing() {
        let pos_x_camera = Camera {
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        let neg_x_camera = Camera {
            swing: CameraSwing::NegX,
            ..Camera::default()
        };

        let pos_x = project_world_to_camera_units(pos_x_camera, WorldPoint { x: 2, y: 0, z: 0 });
        let neg_x = project_world_to_camera_units(neg_x_camera, WorldPoint { x: 2, y: 0, z: 0 });

        assert!(pos_x[0] < 0.0);
        assert!(pos_x[1] < 0.0);
        assert!(neg_x[0] < 0.0);
        assert!(neg_x[1] < 0.0);
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
