use std::ops::RangeInclusive;

use thaum_renderer_domain::{project_world_to_view_plane, Camera, WorldPoint};

pub fn visible_depth_range_for_world_points(
    camera: Camera,
    world_points: &[WorldPoint],
) -> Option<RangeInclusive<i32>> {
    let mut depths = world_points
        .iter()
        .map(|world| project_world_to_view_plane(camera, *world).plane);

    let first = depths.next()?;
    let mut min_depth = first;
    let mut max_depth = first;

    for depth in depths {
        min_depth = min_depth.min(depth);
        max_depth = max_depth.max(depth);
    }

    Some(min_depth..=max_depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use thaum_renderer_domain::CameraSwing;

    #[test]
    fn visible_depth_range_is_relative_to_camera_focus_plane() {
        let camera = Camera {
            position: WorldPoint { x: 9, y: 9, z: 9 },
            focus_target: WorldPoint { x: 2, y: -3, z: 5 },
            swing: CameraSwing::PosZ,
            ..Camera::default()
        };
        let world_points = [
            WorldPoint { x: 0, y: 0, z: 3 },
            WorldPoint { x: 1, y: 4, z: 5 },
            WorldPoint { x: 2, y: 9, z: 8 },
        ];

        assert_eq!(
            visible_depth_range_for_world_points(camera, &world_points),
            Some(-2..=3)
        );
    }

    #[test]
    fn visible_depth_range_tracks_active_depth_axis_from_swing() {
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
            visible_depth_range_for_world_points(camera, &world_points),
            Some(-2..=3)
        );
    }

    #[test]
    fn visible_depth_range_returns_none_for_empty_world_points() {
        assert_eq!(
            visible_depth_range_for_world_points(Camera::default(), &[]),
            None
        );
    }
}
