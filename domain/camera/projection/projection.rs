use crate::{Camera, CameraSwing, CellGroupIntakeBehavior, CellPoint, WorldPoint};

use super::view_orientation::{
    camera_view_orientation_for_camera, camera_view_orientation_for_swing,
    project_world_relative_to_view, unproject_view_relative_to_world, ViewRelativePoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjectionMode {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraProjectedPoint {
    pub u: f32,
    pub v: f32,
    pub plane: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisiblePlaneStack {
    pub focus_plane: i32,
    pub min_plane: i32,
    pub max_plane: i32,
    pub planes: Vec<i32>,
}

pub fn focus_plane_for_camera(_camera: Camera) -> i32 {
    0
}

pub fn build_visible_plane_stack_around_focus(visible_plane_radius: i32) -> VisiblePlaneStack {
    build_visible_plane_stack_with_depth_offset(visible_plane_radius, 0)
}

pub fn build_visible_plane_stack_with_depth_offset(
    visible_plane_radius: i32,
    visible_plane_depth_offset: i32,
) -> VisiblePlaneStack {
    let radius = visible_plane_radius.max(0);
    let min_plane = (-radius + visible_plane_depth_offset).min(0);
    let max_plane = (radius + visible_plane_depth_offset).max(0);

    VisiblePlaneStack {
        focus_plane: 0,
        min_plane,
        max_plane,
        planes: (min_plane..=max_plane).collect(),
    }
}

pub fn visible_plane_stack_for_camera(camera: Camera) -> VisiblePlaneStack {
    build_visible_plane_stack_with_depth_offset(
        camera.visible_plane_radius,
        camera.visible_plane_depth_offset,
    )
}

pub fn projected_plane_is_visible(camera: Camera, plane: i32) -> bool {
    let stack = visible_plane_stack_for_camera(camera);
    plane >= stack.min_plane && plane <= stack.max_plane
}

const ROTATING_3D_PERSPECTIVE_STRENGTH: f32 = 0.5;
const FLAT_2D_LOCAL_DEPTH_STRENGTH: f32 = 0.18;
const PERSPECTIVE_DEPTH_EASE_POWER: f32 = 0.72;
const PERSPECTIVE_PLANE_SPREAD_STRENGTH: f32 = 0.035;

fn eased_signed_depth_units(depth: i32) -> f32 {
    if depth == 0 {
        return 0.0;
    }

    let sign = depth.signum() as f32;
    let magnitude = ((depth.abs() as f32) + 1.0).powf(PERSPECTIVE_DEPTH_EASE_POWER) - 1.0;
    sign * magnitude
}

fn authored_depth_screen_direction_for_swing(swing: CameraSwing) -> [f32; 2] {
    match swing {
        CameraSwing::PosZ => [0.72, -0.72],
        CameraSwing::PosX => [-0.72, -0.72],
        CameraSwing::NegZ => [-0.72, 0.72],
        CameraSwing::NegX => [0.72, 0.72],
        CameraSwing::PosY => [0.42, -1.0],
        CameraSwing::NegY => [-0.42, 1.0],
    }
}

fn curved_depth_screen_offset(depth: i32, direction: [f32; 2], strength: f32) -> [f32; 2] {
    let magnitude = strength * eased_signed_depth_units(depth);

    [direction[0] * magnitude, direction[1] * magnitude]
}

fn perspective_plane_spread_factor(depth: i32) -> f32 {
    let signed_magnitude = PERSPECTIVE_PLANE_SPREAD_STRENGTH * eased_signed_depth_units(depth);

    if signed_magnitude < 0.0 {
        1.0 + signed_magnitude.abs()
    } else {
        1.0 / (1.0 + signed_magnitude)
    }
}

fn focus_plane_local_depth_screen_direction() -> [f32; 2] {
    [0.5, -0.5]
}

pub fn derive_visible_plane_stack_from_world_points(
    camera: Camera,
    world_points: &[WorldPoint],
) -> Option<VisiblePlaneStack> {
    let mut planes = world_points
        .iter()
        .map(|world| project_world_to_view_plane(camera, *world).plane)
        .collect::<Vec<_>>();

    if planes.is_empty() {
        return None;
    }

    planes.sort_unstable();
    planes.dedup();

    let focus_plane = focus_plane_for_camera(camera);
    let min_plane = planes[0];
    let max_plane = *planes.last().unwrap();

    Some(VisiblePlaneStack {
        focus_plane,
        min_plane,
        max_plane,
        planes,
    })
}

pub fn project_world_to_view_plane(camera: Camera, world: WorldPoint) -> CameraProjectedPoint {
    project_world_to_view_plane_for_intake(camera, world, CellGroupIntakeBehavior::Rotating3d)
}

pub fn project_world_to_view_plane_for_intake(
    camera: Camera,
    world: WorldPoint,
    intake_behavior: CellGroupIntakeBehavior,
) -> CameraProjectedPoint {
    match intake_behavior {
        CellGroupIntakeBehavior::Rotating3d => {
            project_rotating_3d_world_to_view_plane(camera, world)
        }
        CellGroupIntakeBehavior::Flat2d => {
            project_flat_2d_world_to_view_plane(camera, world, CellPoint::origin())
        }
    }
}

pub fn project_rotating_3d_world_to_view_plane(
    camera: Camera,
    world: WorldPoint,
) -> CameraProjectedPoint {
    let orientation = camera_view_orientation_for_camera(camera.swing, camera.roll);
    let relative = project_world_relative_to_view(orientation, camera.focus_target, world);
    let (depth_offset, plane_spread) = match camera.projection_mode {
        CameraProjectionMode::Perspective => (
            curved_depth_screen_offset(
                relative.depth,
                authored_depth_screen_direction_for_swing(camera.swing),
                ROTATING_3D_PERSPECTIVE_STRENGTH,
            ),
            perspective_plane_spread_factor(relative.depth),
        ),
        CameraProjectionMode::Orthographic => ([0.0, 0.0], 1.0),
    };

    CameraProjectedPoint {
        u: relative.right as f32 * plane_spread + depth_offset[0],
        v: relative.up as f32 * plane_spread + depth_offset[1],
        plane: relative.depth,
    }
}

pub fn project_flat_2d_world_to_view_plane(
    camera: Camera,
    anchor_world: WorldPoint,
    local: CellPoint,
) -> CameraProjectedPoint {
    let orientation = camera_view_orientation_for_swing(camera.swing);
    let relative = project_world_relative_to_view(orientation, camera.focus_target, anchor_world);
    let (anchor_depth_offset, anchor_plane_spread) = match camera.projection_mode {
        CameraProjectionMode::Perspective => (
            curved_depth_screen_offset(
                relative.depth,
                authored_depth_screen_direction_for_swing(camera.swing),
                ROTATING_3D_PERSPECTIVE_STRENGTH,
            ),
            perspective_plane_spread_factor(relative.depth),
        ),
        CameraProjectionMode::Orthographic => ([0.0, 0.0], 1.0),
    };
    let local_depth_offset = match camera.projection_mode {
        CameraProjectionMode::Perspective => curved_depth_screen_offset(
            local.z,
            focus_plane_local_depth_screen_direction(),
            FLAT_2D_LOCAL_DEPTH_STRENGTH,
        ),
        CameraProjectionMode::Orthographic => [0.0, 0.0],
    };

    CameraProjectedPoint {
        u: relative.right as f32 * anchor_plane_spread
            + anchor_depth_offset[0]
            + local.x as f32
            + local_depth_offset[0],
        v: relative.up as f32 * anchor_plane_spread
            + anchor_depth_offset[1]
            + local.y as f32
            + local_depth_offset[1],
        plane: relative.depth,
    }
}

pub fn unproject_view_plane_to_world(
    camera: Camera,
    projected: CameraProjectedPoint,
) -> WorldPoint {
    let orientation = camera_view_orientation_for_camera(camera.swing, camera.roll);
    let plane = projected.plane;
    let (depth_offset, plane_spread) = match camera.projection_mode {
        CameraProjectionMode::Perspective => (
            curved_depth_screen_offset(
                plane,
                authored_depth_screen_direction_for_swing(camera.swing),
                ROTATING_3D_PERSPECTIVE_STRENGTH,
            ),
            perspective_plane_spread_factor(plane),
        ),
        CameraProjectionMode::Orthographic => ([0.0, 0.0], 1.0),
    };
    let right = ((projected.u - depth_offset[0]) / plane_spread).round() as i32;
    let up = ((projected.v - depth_offset[1]) / plane_spread).round() as i32;

    unproject_view_relative_to_world(
        orientation,
        camera.focus_target,
        ViewRelativePoint {
            right,
            up,
            depth: plane,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CameraSwing;

    #[test]
    fn focus_plane_is_zero_in_camera_relative_projection_space() {
        assert_eq!(focus_plane_for_camera(Camera::default()), 0);
    }

    #[test]
    fn visible_plane_stack_covers_focus_centered_radius() {
        let stack = build_visible_plane_stack_around_focus(2);

        assert_eq!(stack.focus_plane, 0);
        assert_eq!(stack.min_plane, -2);
        assert_eq!(stack.max_plane, 2);
        assert_eq!(stack.planes, vec![-2, -1, 0, 1, 2]);
    }

    #[test]
    fn project_world_to_view_plane_offsets_right_and_up_by_curved_depth() {
        let camera = Camera::default();
        let projected = project_world_to_view_plane(camera, WorldPoint { x: 0, y: 0, z: 2 });

        assert!(projected.u > 0.0);
        assert!(projected.v < 0.0);
        assert!((projected.u + projected.v).abs() < 0.01);
        assert_eq!(projected.plane, 2);
    }

    #[test]
    fn rotating_3d_depth_offset_rotates_with_authored_horizontal_views() {
        let world = WorldPoint { x: 2, y: 0, z: 0 };

        let pos_x = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosX,
                ..Camera::default()
            },
            world,
        );
        let neg_x = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::NegX,
                ..Camera::default()
            },
            world,
        );

        assert!(pos_x.u < 0.0);
        assert!(pos_x.v < 0.0);
        assert_eq!(pos_x.plane, 2);
        assert!(neg_x.u < 0.0);
        assert!(neg_x.v < 0.0);
        assert_eq!(neg_x.plane, -2);
    }

    #[test]
    fn vertical_swings_keep_the_same_depth_push_but_in_a_different_direction() {
        let top = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosY,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 2, z: 0 },
        );
        let bottom = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::NegY,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 2, z: 0 },
        );

        assert!(top.u > 0.0);
        assert!(top.v < 0.0);
        assert_eq!(top.plane, 2);
        assert!(bottom.u > 0.0);
        assert!(bottom.v < 0.0);
        assert_eq!(bottom.plane, -2);
    }

    #[test]
    fn projection_round_trips_world_points() {
        let camera = Camera {
            focus_target: WorldPoint { x: 4, y: -1, z: 8 },
            swing: CameraSwing::NegX,
            ..Camera::default()
        };
        let world = WorldPoint { x: 1, y: 5, z: 10 };

        assert_eq!(
            unproject_view_plane_to_world(camera, project_world_to_view_plane(camera, world)),
            world
        );
    }

    #[test]
    fn flat_2d_intake_uses_shared_anchor_but_keeps_local_screen_axes() {
        let camera = Camera {
            focus_target: WorldPoint { x: 4, y: -3, z: 9 },
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        let anchor_world = WorldPoint { x: 6, y: 2, z: 12 };
        let projected = project_flat_2d_world_to_view_plane(
            camera,
            anchor_world,
            CellPoint { x: 3, y: -4, z: 1 },
        );
        let anchor = project_flat_2d_world_to_view_plane(camera, anchor_world, CellPoint::origin());

        assert!(projected.u > anchor.u + 2.5);
        assert!(projected.v < anchor.v - 3.5);
        assert_eq!(projected.plane, 2);
    }

    #[test]
    fn curved_depth_eases_the_first_steps_but_still_grows_with_distance() {
        let near = project_world_to_view_plane(Camera::default(), WorldPoint { x: 0, y: 0, z: 1 });
        let mid = project_world_to_view_plane(Camera::default(), WorldPoint { x: 0, y: 0, z: 2 });
        let far = project_world_to_view_plane(Camera::default(), WorldPoint { x: 0, y: 0, z: 3 });

        assert!(near.u > 0.0);
        assert!(mid.u > near.u);
        assert!(far.u > mid.u);
        assert!(far.u < near.u * 3.0);
    }

    #[test]
    fn plane_spread_expands_toward_camera_and_contracts_away_from_camera() {
        assert!(perspective_plane_spread_factor(-3) > 1.0);
        assert!(perspective_plane_spread_factor(3) < 1.0);
        assert!(perspective_plane_spread_factor(-3) > perspective_plane_spread_factor(-1));
        assert!(perspective_plane_spread_factor(3) < perspective_plane_spread_factor(1));
    }

    #[test]
    fn flat_2d_anchor_keeps_shared_world_perspective_alignment_with_rotating_3d() {
        let camera = Camera {
            focus_target: WorldPoint { x: 0, y: 0, z: 0 },
            swing: CameraSwing::PosZ,
            ..Camera::default()
        };
        let world = WorldPoint { x: 3, y: 2, z: 2 };

        let rotating = project_rotating_3d_world_to_view_plane(camera, world);
        let flat = project_flat_2d_world_to_view_plane(camera, world, CellPoint::origin());

        assert_eq!(rotating, flat);
    }

    #[test]
    fn flat_2d_intake_does_not_rotate_local_typegrid_offsets_with_roll() {
        let projected = project_flat_2d_world_to_view_plane(
            Camera::default(),
            WorldPoint::origin(),
            CellPoint { x: 2, y: 1, z: 0 },
        );

        assert_eq!((projected.u, projected.v, projected.plane), (2.0, 1.0, 0));
    }

    #[test]
    fn flat_2d_anchor_projection_is_not_affected_by_camera_roll() {
        let unrolled = project_flat_2d_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosZ,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 0, z: 2 },
            CellPoint::origin(),
        );
        let rolled = project_flat_2d_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosZ,
                roll: crate::CameraRoll::Deg90,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 0, z: 2 },
            CellPoint::origin(),
        );

        assert_eq!(unrolled, rolled);
    }

    #[test]
    fn orthographic_projection_stacks_same_xy_directly_over_each_other() {
        let camera = Camera {
            projection_mode: CameraProjectionMode::Orthographic,
            ..Camera::default()
        };

        let a = project_world_to_view_plane(camera, WorldPoint { x: 0, y: 0, z: 0 });
        let b = project_world_to_view_plane(camera, WorldPoint { x: 0, y: 0, z: 3 });

        assert_eq!((a.u, a.v), (0.0, 0.0));
        assert_eq!((b.u, b.v), (0.0, 0.0));
        assert_eq!(b.plane, 3);
    }

    #[test]
    fn derive_visible_plane_stack_collects_projected_planes_from_world_points() {
        let camera = Camera {
            focus_target: WorldPoint { x: 5, y: 0, z: 9 },
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        let world_points = [
            WorldPoint { x: 3, y: 2, z: 99 },
            WorldPoint {
                x: 5,
                y: -1,
                z: -20,
            },
            WorldPoint { x: 8, y: 4, z: 7 },
        ];

        assert_eq!(
            derive_visible_plane_stack_from_world_points(camera, &world_points),
            Some(VisiblePlaneStack {
                focus_plane: 0,
                min_plane: -2,
                max_plane: 3,
                planes: vec![-2, 0, 3],
            })
        );
    }
}
