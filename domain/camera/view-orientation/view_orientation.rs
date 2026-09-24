use crate::{
    cell_facing::{CellFacing, CellRoll, FacingRotation},
    AxisSign, GlobalDirection, WorldAxis, WorldPoint,
};

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

/// The camera-facing carrier for relative-facing resolution: the world side
/// the camera views FROM (the opposite of its look direction), as a facing.
/// This convention makes `CellFacing::relative_facing` return `PosZ` exactly
/// when the viewer sees a cell's front, so authored front art resolves
/// naturally. Camera roll is display-space rotation around the depth axis and
/// does not participate in facing resolution — a rolled screen still looks at
/// the same side of an object.
pub const fn camera_facing_for_swing(swing: CameraSwing) -> CellFacing {
    CellFacing::from_global_direction(opposite_direction(
        camera_view_orientation_for_swing(swing).depth,
    ))
}

/// The composed relative facing of one cell under one group facing as seen
/// from this camera swing: the single variant-lookup key component. `PosZ`
/// means the cell's front is visible.
pub fn view_relative_facing(swing: CameraSwing, cell: CellFacing, group: CellFacing) -> CellFacing {
    FacingRotation::relative_rotation(
        FacingRotation::from_facing(cell),
        FacingRotation::from_facing(group),
        camera_rotation_for_camera(swing, CameraRoll::Deg0),
    )
    .facing
}

/// The camera's full orientation as a rotation in the 24-element group: the
/// unrolled view-from orientation composed with its roll on the canonical
/// side (the camera rolls around its look axis, which is the reverse of the
/// facing frame's z, so the roll is complemented; pinned by test).
pub fn camera_rotation_for_camera(swing: CameraSwing, roll: CameraRoll) -> FacingRotation {
    FacingRotation::from_facing(camera_facing_for_swing(swing)).compose(FacingRotation::new(
        CellFacing::PosZ,
        camera_roll_as_cell_roll(roll),
    ))
}

const fn camera_roll_as_cell_roll(roll: CameraRoll) -> CellRoll {
    match roll {
        CameraRoll::Deg0 => CellRoll::Deg0,
        CameraRoll::Deg90 => CellRoll::Deg270,
        CameraRoll::Deg180 => CellRoll::Deg180,
        CameraRoll::Deg270 => CellRoll::Deg90,
    }
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
    fn camera_facing_is_the_side_the_camera_views_from() {
        // Default swing looks along South (+z), so it views from North.
        assert_eq!(camera_facing_for_swing(CameraSwing::PosZ), CellFacing::NegZ);
        // An East-looking camera views from the West side.
        assert_eq!(camera_facing_for_swing(CameraSwing::PosX), CellFacing::NegX);
        assert_eq!(camera_facing_for_swing(CameraSwing::PosY), CellFacing::NegY);
        assert_eq!(camera_facing_for_swing(CameraSwing::NegY), CellFacing::PosY);
    }

    #[test]
    fn front_facing_cells_resolve_to_posz_from_the_default_view() {
        // Default camera views from North: a cell fronting North shows its
        // front, one fronting South shows its back.
        assert_eq!(
            view_relative_facing(CameraSwing::PosZ, CellFacing::NegZ, CellFacing::PosZ),
            CellFacing::PosZ
        );
        assert_eq!(
            view_relative_facing(CameraSwing::PosZ, CellFacing::PosZ, CellFacing::PosZ),
            CellFacing::NegZ
        );
    }

    #[test]
    fn front_view_tracks_the_camera_swing() {
        // A cell fronting West shows its front from the PosX swing (which
        // views from West) and its back once the camera swings to NegX
        // (viewing from East).
        assert_eq!(
            view_relative_facing(CameraSwing::PosX, CellFacing::NegX, CellFacing::PosZ),
            CellFacing::PosZ
        );
        assert_eq!(
            view_relative_facing(CameraSwing::NegX, CellFacing::NegX, CellFacing::PosZ),
            CellFacing::NegZ
        );
    }

    #[test]
    fn group_facing_turns_what_the_viewer_sees() {
        // A PosZ-fronting cell on a PosX-facing group shows a side view from
        // the default view: the group rotation turned the front East, which
        // the North-viewing camera sees from the side (frame -x).
        assert_eq!(
            view_relative_facing(CameraSwing::PosZ, CellFacing::PosZ, CellFacing::PosX),
            CellFacing::NegX
        );
        // Swing the camera to view from the East side and the front returns.
        assert_eq!(
            view_relative_facing(CameraSwing::NegX, CellFacing::PosZ, CellFacing::PosX),
            CellFacing::PosZ
        );
    }

    #[test]
    fn camera_roll_lands_in_the_relative_roll_component_one_to_one() {
        // Default swing (views from North), cell fronting North on an
        // identity group: front view at every camera roll, and the camera
        // roll lands one-to-one as the relative rotation's roll component.
        for &camera_roll in &[
            CameraRoll::Deg0,
            CameraRoll::Deg90,
            CameraRoll::Deg180,
            CameraRoll::Deg270,
        ] {
            let relative = FacingRotation::relative_rotation(
                FacingRotation::from_facing(CellFacing::NegZ),
                FacingRotation::IDENTITY,
                camera_rotation_for_camera(CameraSwing::PosZ, camera_roll),
            );
            assert_eq!(relative.facing, CellFacing::PosZ);
            assert_eq!(relative.roll, camera_roll_as_cell_roll(camera_roll));
        }
    }

    #[test]
    fn roll_never_changes_which_side_of_a_cell_is_visible() {
        // Rolling the screen never swaps which side of an object is seen:
        // the side is the unrolled relative facing at every camera roll.
        for &roll in &[
            CameraRoll::Deg0,
            CameraRoll::Deg90,
            CameraRoll::Deg180,
            CameraRoll::Deg270,
        ] {
            for &cell in &ALL_FACINGS {
                let unrolled = view_relative_facing(CameraSwing::PosZ, cell, CellFacing::PosZ);
                assert_eq!(baseline_of(cell).facing, unrolled);
                let relative = FacingRotation::relative_rotation(
                    FacingRotation::from_facing(cell),
                    FacingRotation::IDENTITY,
                    camera_rotation_for_camera(CameraSwing::PosZ, roll),
                );
                // The side survives the roll untouched; the camera roll
                // multiplies the relative roll tag only (image rotation).
                assert_eq!(
                    relative,
                    baseline_of(cell).compose(FacingRotation::new(
                        CellFacing::PosZ,
                        camera_roll_as_cell_roll(roll)
                    ))
                );
            }
        }
    }

    fn baseline_of(cell: CellFacing) -> FacingRotation {
        FacingRotation::relative_rotation(
            FacingRotation::from_facing(cell),
            FacingRotation::IDENTITY,
            FacingRotation::from_facing(camera_facing_for_swing(CameraSwing::PosZ)),
        )
    }

    const ALL_FACINGS: [CellFacing; 6] = [
        CellFacing::PosX,
        CellFacing::NegX,
        CellFacing::PosY,
        CellFacing::NegY,
        CellFacing::PosZ,
        CellFacing::NegZ,
    ];

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
