use crate::{Camera, WorldPoint};

use super::projection::{unproject_view_plane_to_world, CameraProjectedPoint};

pub fn remap_camera_units_to_active_plane_world(
    camera: Camera,
    camera_units: [f32; 2],
) -> WorldPoint {
    remap_camera_units_to_world_on_plane(camera, camera_units, 0)
}

pub fn remap_camera_units_to_world_on_plane(
    camera: Camera,
    camera_units: [f32; 2],
    plane: i32,
) -> WorldPoint {
    unproject_view_plane_to_world(
        camera,
        CameraProjectedPoint {
            u: camera_units[0],
            v: camera_units[1],
            plane,
        },
    )
}

pub fn remap_surface_units_to_active_plane_world(
    camera: Camera,
    surface_units: [f32; 2],
    cell_clip_size: [f32; 2],
) -> WorldPoint {
    remap_camera_units_to_active_plane_world(
        camera,
        [
            surface_units[0] / cell_clip_size[0],
            surface_units[1] / cell_clip_size[1],
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CameraSwing;

    use super::super::projection::project_world_to_view_plane;

    #[test]
    fn remap_active_plane_world_round_trips_focus_plane_cells() {
        let camera = Camera {
            focus_target: WorldPoint { x: 3, y: -2, z: 4 },
            swing: CameraSwing::PosZ,
            ..Camera::default()
        };
        let world = WorldPoint { x: 6, y: 1, z: 4 };
        let projected = project_world_to_view_plane(camera, world);

        assert_eq!(projected.plane, 0);
        assert_eq!(
            remap_camera_units_to_active_plane_world(camera, [projected.u, projected.v]),
            world
        );
    }

    #[test]
    fn remap_world_on_requested_plane_round_trips_non_focus_plane_cells() {
        let camera = Camera {
            focus_target: WorldPoint::origin(),
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        let world = WorldPoint { x: 2, y: 5, z: -4 };
        let projected = project_world_to_view_plane(camera, world);

        assert_eq!(
            remap_camera_units_to_world_on_plane(
                camera,
                [projected.u, projected.v],
                projected.plane
            ),
            world
        );
    }

    #[test]
    fn remap_surface_units_divides_by_cell_clip_size_before_unprojection() {
        let camera = Camera::default();
        let world = WorldPoint { x: 3, y: 2, z: 0 };
        let projected = project_world_to_view_plane(camera, world);
        let cell_clip_size = [0.25, 0.5];
        let surface_units = [
            projected.u * cell_clip_size[0],
            projected.v * cell_clip_size[1],
        ];

        assert_eq!(
            remap_surface_units_to_active_plane_world(camera, surface_units, cell_clip_size),
            world
        );
    }
}
