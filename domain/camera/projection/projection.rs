use crate::{Camera, CellGroupIntakeBehavior, CellPoint, WorldPoint};

use super::perspective::{depth_position_spread, depth_scale_factor};
use super::parallax::parallax_screen_offset;

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

const FLAT_2D_LOCAL_DEPTH_STRENGTH: f32 = 0.16;

fn curved_depth_screen_offset(depth: i32, direction: [f32; 2], strength: f32) -> [f32; 2] {
    let magnitude = strength * super::perspective::eased_signed_depth_units(depth);

    [direction[0] * magnitude, direction[1] * magnitude]
}

/// Glyph scale factor for one depth unit under the camera's perspective
/// profile. Position spread is the separate `depth_position_spread`.
pub fn projected_plane_scale_factor(camera: Camera, depth: i32) -> f32 {
    depth_scale_factor(depth, camera.projection_mode, camera.perspective)
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
    let plane_spread = depth_position_spread(relative.depth, camera.projection_mode, camera.perspective);
    let parallax = parallax_screen_offset(camera.parallax, relative.depth, camera.projection_mode);

    CameraProjectedPoint {
        u: relative.right as f32 * plane_spread + parallax[0],
        v: relative.up as f32 * plane_spread + parallax[1],
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
    let anchor_plane_spread =
        depth_position_spread(relative.depth, camera.projection_mode, camera.perspective);
    let local_depth_offset = match camera.projection_mode {
        CameraProjectionMode::Perspective => curved_depth_screen_offset(
            local.z,
            focus_plane_local_depth_screen_direction(),
            FLAT_2D_LOCAL_DEPTH_STRENGTH,
        ),
        CameraProjectionMode::Orthographic => [0.0, 0.0],
    };

    CameraProjectedPoint {
        u: relative.right as f32 * anchor_plane_spread + local.x as f32 + local_depth_offset[0],
        v: relative.up as f32 * anchor_plane_spread + local.y as f32 + local_depth_offset[1],
        plane: relative.depth,
    }
}

/// Inverse of `project_flat_2d_world_to_view_plane` when the anchor is always
/// `camera.focus_target` (the only anchor the boot pipeline uses) — the
/// anchor's relative offset is always zero, so this needs none of
/// `camera`'s swing/roll/focus_target, only its own HUD pan offset.
pub fn unproject_flat_2d_view_plane_to_local(
    camera: Camera,
    projected: CameraProjectedPoint,
) -> CellPoint {
    CellPoint {
        x: projected.u.round() as i32 - camera.hud_pan_offset.x,
        y: projected.v.round() as i32 - camera.hud_pan_offset.y,
        z: 0,
    }
}

pub fn unproject_view_plane_to_world(
    camera: Camera,
    projected: CameraProjectedPoint,
) -> WorldPoint {
    let orientation = camera_view_orientation_for_camera(camera.swing, camera.roll);
    let plane = projected.plane;
    let plane_spread = depth_position_spread(plane, camera.projection_mode, camera.perspective);
    // Exact inverse of the projection's parallax addition: remove it before
    // dividing, so world round-trips hold while parallax is live.
    let parallax = parallax_screen_offset(camera.parallax, plane, camera.projection_mode);
    let right = ((projected.u - parallax[0]) / plane_spread).round() as i32;
    let up = ((projected.v - parallax[1]) / plane_spread).round() as i32;

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
    fn focus_target_projects_to_perspective_convergence_point() {
        let camera = Camera::default();
        let projected = project_world_to_view_plane(camera, camera.focus_target);

        assert_eq!(
            projected,
            CameraProjectedPoint {
                u: 0.0,
                v: 0.0,
                plane: 0
            }
        );
    }

    #[test]
    fn perspective_scales_screen_offset_toward_focus_target_without_extra_drift() {
        let near = project_world_to_view_plane(Camera::default(), WorldPoint { x: 2, y: 0, z: -2 });
        let far = project_world_to_view_plane(Camera::default(), WorldPoint { x: 2, y: 0, z: 2 });

        assert!(near.u > far.u);
        assert_eq!(near.v, 0.0);
        assert_eq!(far.v, 0.0);
    }

    #[test]
    fn all_swings_keep_zero_right_up_points_on_the_focus_convergence_axis() {
        let pos_x = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosX,
                ..Camera::default()
            },
            WorldPoint { x: 2, y: 0, z: 0 },
        );
        let neg_x = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::NegX,
                ..Camera::default()
            },
            WorldPoint { x: 2, y: 0, z: 0 },
        );
        let pos_y = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::PosY,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 2, z: 0 },
        );
        let neg_y = project_world_to_view_plane(
            Camera {
                swing: CameraSwing::NegY,
                ..Camera::default()
            },
            WorldPoint { x: 0, y: 2, z: 0 },
        );

        assert_eq!((pos_x.u, pos_x.v, pos_x.plane), (0.0, 0.0, 2));
        assert_eq!((neg_x.u, neg_x.v, neg_x.plane), (0.0, 0.0, -2));
        assert_eq!((pos_y.u, pos_y.v, pos_y.plane), (0.0, 0.0, 2));
        assert_eq!((neg_y.u, neg_y.v, neg_y.plane), (0.0, 0.0, -2));
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
        let camera = Camera::default();
        let near = projected_plane_scale_factor(camera, 1);
        let mid = projected_plane_scale_factor(camera, 2);
        let far = projected_plane_scale_factor(camera, 3);

        assert!(near < 1.0);
        assert!(mid < near);
        assert!(far < mid);
        assert!(far > near / 3.0);
    }

    #[test]
    fn plane_spread_expands_toward_camera_and_contracts_away_from_camera() {
        let camera = Camera::default();
        assert!(projected_plane_scale_factor(camera, -3) > 1.0);
        assert!(projected_plane_scale_factor(camera, 3) < 1.0);
        assert!(projected_plane_scale_factor(camera, -3)
            > projected_plane_scale_factor(camera, -1));
        assert!(projected_plane_scale_factor(camera, 3)
            < projected_plane_scale_factor(camera, 1));
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
    fn unproject_flat_2d_view_plane_to_local_round_trips_projected_local_offsets() {
        let local = CellPoint { x: -6, y: 12, z: 0 };
        let camera = Camera {
            focus_target: WorldPoint {
                x: 40,
                y: -17,
                z: 6,
            },
            swing: CameraSwing::PosX,
            roll: crate::CameraRoll::Deg90,
            ..Camera::default()
        };
        let projected = project_flat_2d_world_to_view_plane(camera, camera.focus_target, local);

        assert_eq!(
            unproject_flat_2d_view_plane_to_local(camera, projected),
            local
        );
    }

    #[test]
    fn unproject_flat_2d_view_plane_to_local_reverses_hud_pan_offset() {
        let camera = Camera {
            hud_pan_offset: CellPoint { x: 5, y: -3, z: 0 },
            ..Camera::default()
        };

        assert_eq!(
            unproject_flat_2d_view_plane_to_local(
                camera,
                CameraProjectedPoint {
                    u: 5.0,
                    v: -3.0,
                    plane: 0,
                },
            ),
            CellPoint::origin()
        );
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
