use crate::{AxisSign, GlobalDirection, WorldAxis, WorldPoint};

use super::{roll::CameraRoll, swing::CameraSwing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraViewOrientation {
    pub depth: GlobalDirection,
    pub right: GlobalDirection,
    pub up: GlobalDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRelativePoint {
    pub right: i32,
    pub up: i32,
    pub depth: i32,
}

pub const fn camera_view_orientation_for_swing(swing: CameraSwing) -> CameraViewOrientation {
    match swing {
        CameraSwing::PosZ => CameraViewOrientation {
            depth: GlobalDirection::South,
            right: GlobalDirection::East,
            up: GlobalDirection::Top,
        },
        CameraSwing::PosX => CameraViewOrientation {
            depth: GlobalDirection::East,
            right: GlobalDirection::North,
            up: GlobalDirection::Top,
        },
        CameraSwing::NegZ => CameraViewOrientation {
            depth: GlobalDirection::North,
            right: GlobalDirection::West,
            up: GlobalDirection::Top,
        },
        CameraSwing::NegX => CameraViewOrientation {
            depth: GlobalDirection::West,
            right: GlobalDirection::South,
            up: GlobalDirection::Top,
        },
        CameraSwing::PosY => CameraViewOrientation {
            depth: GlobalDirection::Top,
            right: GlobalDirection::East,
            up: GlobalDirection::North,
        },
        CameraSwing::NegY => CameraViewOrientation {
            depth: GlobalDirection::Bottom,
            right: GlobalDirection::East,
            up: GlobalDirection::South,
        },
    }
}

pub const fn camera_view_orientation_for_camera(
    swing: CameraSwing,
    roll: CameraRoll,
) -> CameraViewOrientation {
    let base = camera_view_orientation_for_swing(swing);

    match roll {
        CameraRoll::Deg0 => base,
        CameraRoll::Deg90 => CameraViewOrientation {
            depth: base.depth,
            right: base.up,
            up: opposite_direction(base.right),
        },
        CameraRoll::Deg180 => CameraViewOrientation {
            depth: base.depth,
            right: opposite_direction(base.right),
            up: opposite_direction(base.up),
        },
        CameraRoll::Deg270 => CameraViewOrientation {
            depth: base.depth,
            right: opposite_direction(base.up),
            up: base.right,
        },
    }
}

pub const fn active_depth_axis_for_swing(swing: CameraSwing) -> WorldAxis {
    camera_view_orientation_for_swing(swing).depth.axis()
}

pub const fn active_depth_direction_for_swing(swing: CameraSwing) -> GlobalDirection {
    camera_view_orientation_for_swing(swing).depth
}

pub fn project_world_relative_to_view(
    orientation: CameraViewOrientation,
    focus_target: WorldPoint,
    world: WorldPoint,
) -> ViewRelativePoint {
    let relative = WorldPoint {
        x: world.x - focus_target.x,
        y: world.y - focus_target.y,
        z: world.z - focus_target.z,
    };

    ViewRelativePoint {
        right: project_onto_direction(relative, orientation.right),
        up: project_onto_direction(relative, orientation.up),
        depth: project_onto_direction(relative, orientation.depth),
    }
}

pub fn unproject_view_relative_to_world(
    orientation: CameraViewOrientation,
    focus_target: WorldPoint,
    view: ViewRelativePoint,
) -> WorldPoint {
    let right = direction_unit_vector(orientation.right);
    let up = direction_unit_vector(orientation.up);
    let depth = direction_unit_vector(orientation.depth);

    WorldPoint {
        x: focus_target.x + right[0] * view.right + up[0] * view.up + depth[0] * view.depth,
        y: focus_target.y + right[1] * view.right + up[1] * view.up + depth[1] * view.depth,
        z: focus_target.z + right[2] * view.right + up[2] * view.up + depth[2] * view.depth,
    }
}

/// Signed world coordinate of `point` along `direction`'s axis, honoring the
/// direction sign — the user-facing depth read for a camera focus target.
pub const fn world_depth_along_direction(point: WorldPoint, direction: GlobalDirection) -> i32 {
    project_onto_direction(point, direction)
}

const fn project_onto_direction(point: WorldPoint, direction: GlobalDirection) -> i32 {
    match (direction.axis(), direction.sign()) {
        (WorldAxis::X, AxisSign::Positive) => point.x,
        (WorldAxis::X, AxisSign::Negative) => -point.x,
        (WorldAxis::Y, AxisSign::Positive) => point.y,
        (WorldAxis::Y, AxisSign::Negative) => -point.y,
        (WorldAxis::Z, AxisSign::Positive) => point.z,
        (WorldAxis::Z, AxisSign::Negative) => -point.z,
    }
}

const fn opposite_direction(direction: GlobalDirection) -> GlobalDirection {
    match direction {
        GlobalDirection::East => GlobalDirection::West,
        GlobalDirection::West => GlobalDirection::East,
        GlobalDirection::Top => GlobalDirection::Bottom,
        GlobalDirection::Bottom => GlobalDirection::Top,
        GlobalDirection::South => GlobalDirection::North,
        GlobalDirection::North => GlobalDirection::South,
    }
}

const fn direction_unit_vector(direction: GlobalDirection) -> [i32; 3] {
    direction.unit_vector()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_maps_to_expected_depth_directions() {
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::PosX),
            GlobalDirection::East
        );
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::NegX),
            GlobalDirection::West
        );
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::PosY),
            GlobalDirection::Top
        );
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::NegY),
            GlobalDirection::Bottom
        );
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::PosZ),
            GlobalDirection::South
        );
        assert_eq!(
            active_depth_direction_for_swing(CameraSwing::NegZ),
            GlobalDirection::North
        );
    }

    #[test]
    fn swing_maps_to_expected_depth_axes() {
        assert_eq!(active_depth_axis_for_swing(CameraSwing::PosX), WorldAxis::X);
        assert_eq!(active_depth_axis_for_swing(CameraSwing::NegY), WorldAxis::Y);
        assert_eq!(active_depth_axis_for_swing(CameraSwing::PosZ), WorldAxis::Z);
    }

    #[test]
    fn project_world_relative_to_view_uses_authored_basis_for_positive_x() {
        let orientation = camera_view_orientation_for_swing(CameraSwing::PosX);
        let relative = project_world_relative_to_view(
            orientation,
            WorldPoint { x: 1, y: -2, z: 3 },
            WorldPoint { x: 4, y: 5, z: -1 },
        );

        assert_eq!(
            relative,
            ViewRelativePoint {
                right: 4,
                up: 7,
                depth: 3,
            }
        );
    }

    #[test]
    fn roll_rotates_right_and_up_around_depth() {
        let orientation = camera_view_orientation_for_camera(CameraSwing::PosZ, CameraRoll::Deg90);

        assert_eq!(orientation.depth, GlobalDirection::South);
        assert_eq!(orientation.right, GlobalDirection::Top);
        assert_eq!(orientation.up, GlobalDirection::West);
    }

    #[test]
    fn unproject_round_trips_view_relative_points() {
        let orientation = camera_view_orientation_for_swing(CameraSwing::NegZ);
        let focus_target = WorldPoint { x: 5, y: -3, z: 7 };
        let view = ViewRelativePoint {
            right: 2,
            up: 4,
            depth: 6,
        };

        let world = unproject_view_relative_to_world(orientation, focus_target, view);
        assert_eq!(
            project_world_relative_to_view(orientation, focus_target, world),
            view
        );
    }

    #[test]
    fn focus_target_projects_to_view_origin() {
        let orientation = camera_view_orientation_for_swing(CameraSwing::PosZ);
        let focus_target = WorldPoint { x: 3, y: -2, z: 4 };

        assert_eq!(
            project_world_relative_to_view(orientation, focus_target, focus_target),
            ViewRelativePoint {
                right: 0,
                up: 0,
                depth: 0,
            }
        );
    }
}
