use crate::coordinate_space::global_directions::GlobalDirection;
use crate::coordinate_space::CellPoint;

/// The six-cardinal facing vocabulary shared by camera view, cell groups, and
/// individual cells. A facing denotes the world direction a local front (+z)
/// maps to; `PosZ` is the identity.
///
/// Facing composition is additive: a cell's facing is rotated by its group's
/// facing, and the result is expressed in the camera's frame. The algebra only
/// tracks where the front direction ends up, so no roll is ever materialized
/// and every operation is integer table math over six directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CellFacing {
    PosX,
    NegX,
    PosY,
    NegY,
    #[default]
    PosZ,
    NegZ,
}

impl CellFacing {
    /// All six facings in canonical order.
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];

    /// Apply this facing's rotation to a direction. Composing facings means
    /// chaining this: `group.rotate_direction(cell)` rotates the cell's front
    /// by the group's orientation.
    pub fn rotate_direction(self, direction: Self) -> Self {
        let [x, y, z] = direction.unit_vector();
        Self::from_vector(self.remap_vector(x, y, z))
            .expect("rotating a cardinal direction yields a cardinal direction")
    }

    /// Apply this facing's inverse rotation to a direction. Used to express a
    /// world direction in a frame whose rotation is this facing (the camera
    /// side of the relative-facing resolution).
    pub fn unrotate_direction(self, direction: Self) -> Self {
        Self::ALL
            .into_iter()
            .find(|candidate| self.rotate_direction(*candidate) == direction)
            .expect("every direction has a preimage under a facing rotation")
    }

    /// Pure point remap for this facing's rotation. Cell-group positioning
    /// delegates here so all rotation math lives in one encapsulation.
    pub fn remap_point(self, point: CellPoint) -> CellPoint {
        let [x, y, z] = self.remap_vector(point.x, point.y, point.z);
        CellPoint { x, y, z }
    }

    fn remap_vector(self, x: i32, y: i32, z: i32) -> [i32; 3] {
        match self {
            Self::PosX => [z, y, -x],
            Self::NegX => [-z, y, x],
            Self::PosY => [x, z, -y],
            Self::NegY => [x, -z, y],
            Self::PosZ => [x, y, z],
            Self::NegZ => [-x, y, -z],
        }
    }

    fn unit_vector(self) -> [i32; 3] {
        match self {
            Self::PosX => [1, 0, 0],
            Self::NegX => [-1, 0, 0],
            Self::PosY => [0, 1, 0],
            Self::NegY => [0, -1, 0],
            Self::PosZ => [0, 0, 1],
            Self::NegZ => [0, 0, -1],
        }
    }

    fn from_vector([x, y, z]: [i32; 3]) -> Option<Self> {
        match (x, y, z) {
            (1, 0, 0) => Some(Self::PosX),
            (-1, 0, 0) => Some(Self::NegX),
            (0, 1, 0) => Some(Self::PosY),
            (0, -1, 0) => Some(Self::NegY),
            (0, 0, 1) => Some(Self::PosZ),
            (0, 0, -1) => Some(Self::NegZ),
            _ => None,
        }
    }

    /// Map onto the renderer-global direction language owned by
    /// coordinate-space/global-directions.
    pub const fn to_global_direction(self) -> GlobalDirection {
        match self {
            Self::PosX => GlobalDirection::East,
            Self::NegX => GlobalDirection::West,
            Self::PosY => GlobalDirection::Top,
            Self::NegY => GlobalDirection::Bottom,
            Self::PosZ => GlobalDirection::South,
            Self::NegZ => GlobalDirection::North,
        }
    }

    pub const fn from_global_direction(direction: GlobalDirection) -> Self {
        match direction {
            GlobalDirection::East => Self::PosX,
            GlobalDirection::West => Self::NegX,
            GlobalDirection::Top => Self::PosY,
            GlobalDirection::Bottom => Self::NegY,
            GlobalDirection::South => Self::PosZ,
            GlobalDirection::North => Self::NegZ,
        }
    }

    /// The additive relative-facing resolution: the cell's front direction,
    /// rotated by its group's facing, expressed in the camera's frame. A
    /// result of `PosZ` means the front points along the camera frame's +z.
    ///
    /// This is the single seam where camera, group, and cell facing combine;
    /// variant lookup consumes the result.
    pub fn relative_facing(cell: Self, group: Self, camera: Self) -> Self {
        camera.unrotate_direction(group.rotate_direction(cell))
    }
}

/// Camera-style roll around the canonical +z axis. Combined with a facing it
/// forms one of the 24 elements of the cube rotation group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CellRoll {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl CellRoll {
    /// All four rolls in canonical order.
    pub const ALL: [Self; 4] = [Self::Deg0, Self::Deg90, Self::Deg180, Self::Deg270];

    /// Apply this roll's rotation to a direction: a rotation around the
    /// canonical +z axis. `Deg90` maps PosX to PosY (counter-clockwise viewed
    /// from +z).
    pub fn rotate_direction(self, direction: CellFacing) -> CellFacing {
        let [x, y, z] = direction.unit_vector();
        let [x, y, z] = match self {
            Self::Deg0 => [x, y, z],
            Self::Deg90 => [-y, x, z],
            Self::Deg180 => [-x, -y, z],
            Self::Deg270 => [y, -x, z],
        };
        CellFacing::from_vector([x, y, z])
            .expect("rotating a direction around z yields a cardinal direction")
    }

    /// Apply this roll's rotation to a point: z is preserved, xy rotates.
    pub fn rotate_point(self, point: CellPoint) -> CellPoint {
        let (x, y) = match self {
            Self::Deg0 => (point.x, point.y),
            Self::Deg90 => (-point.y, point.x),
            Self::Deg180 => (-point.x, -point.y),
            Self::Deg270 => (point.y, -point.x),
        };
        CellPoint { x, y, z: point.z }
    }
}

/// One full cube rotation: roll the local frame around its own +z, then point
/// it at the facing. This is the 24-element group that facing-only resolution
/// quotients over; `Deg0` roll recovers the plain facing action exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FacingRotation {
    pub facing: CellFacing,
    pub roll: CellRoll,
}

impl FacingRotation {
    pub const IDENTITY: Self = Self {
        facing: CellFacing::PosZ,
        roll: CellRoll::Deg0,
    };

    pub const fn from_facing(facing: CellFacing) -> Self {
        Self {
            facing,
            roll: CellRoll::Deg0,
        }
    }

    pub const fn new(facing: CellFacing, roll: CellRoll) -> Self {
        Self { facing, roll }
    }

    /// Apply this rotation to a canonical direction.
    pub fn rotate_direction(self, direction: CellFacing) -> CellFacing {
        let rolled = self.roll.rotate_direction(direction);
        self.facing.rotate_direction(rolled)
    }

    /// Apply this rotation to a point.
    pub fn rotate_point(self, point: CellPoint) -> CellPoint {
        let rolled = self.roll.rotate_point(point);
        self.facing.remap_point(rolled)
    }

    fn action_on_all_directions(self) -> [CellFacing; 6] {
        let mut actions = [CellFacing::PosZ; 6];
        for (index, &direction) in CellFacing::ALL.iter().enumerate() {
            actions[index] = self.rotate_direction(direction);
        }
        actions
    }

    /// Compose two rotations: `self.compose(other)` applies `other` first.
    /// Resolved by matching the composed action on all six directions, which
    /// uniquely determines an element of the 24-element group; exhaustively
    /// pinned by the group-law tests below.
    pub fn compose(self, other: Self) -> Self {
        let mut composed = [CellFacing::PosZ; 6];
        for (index, &direction) in CellFacing::ALL.iter().enumerate() {
            composed[index] = self.rotate_direction(other.rotate_direction(direction));
        }
        Self::from_action(composed).expect("every action composition is a cube rotation")
    }

    fn from_action(actions: [CellFacing; 6]) -> Option<Self> {
        for &facing in &CellFacing::ALL {
            for &roll in &CellRoll::ALL {
                let candidate = Self { facing, roll };
                if candidate.action_on_all_directions() == actions {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// The inverse rotation.
    pub fn inverse(self) -> Self {
        let mut preimages = [CellFacing::PosZ; 6];
        for (index, &direction) in CellFacing::ALL.iter().enumerate() {
            preimages[index] = CellFacing::ALL
                .into_iter()
                .find(|&candidate| self.rotate_direction(candidate) == direction)
                .expect("every direction has a preimage under a cube rotation");
        }
        Self::from_action(preimages).expect("every preimage table is a cube rotation")
    }

    /// The additive relative-rotation resolution: the cell's orientation,
    /// rotated by its group's orientation, expressed in the camera's frame.
    /// The single variant-lookup key; the facing component alone recovers the
    /// facing-only resolution.
    ///
    /// The camera's roll is an image rotation, not an object rotation: it
    /// rotates the rendered art (a rolled dot is still a dot) without
    /// remapping which face is which. So the camera contributes its swing
    /// facing to the object-side composition, and its roll only multiplies
    /// the final roll tag. Folding the roll into the object composition
    /// instead would let a rolled camera steal a side's key — a stick's end
    /// grain resolving onto a body's lying-profile entry at quarter turns.
    pub fn relative_rotation(cell: Self, group: Self, camera: Self) -> Self {
        let swing = Self::new(camera.facing, CellRoll::Deg0);
        swing
            .inverse()
            .compose(group)
            .compose(cell)
            .compose(Self::new(CellFacing::PosZ, camera.roll))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [CellFacing; 6] = [
        CellFacing::PosX,
        CellFacing::NegX,
        CellFacing::PosY,
        CellFacing::NegY,
        CellFacing::PosZ,
        CellFacing::NegZ,
    ];

    #[test]
    fn default_facing_is_identity() {
        for &direction in &ALL {
            assert_eq!(CellFacing::PosZ.rotate_direction(direction), direction);
            assert_eq!(
                CellFacing::PosZ.remap_point(CellPoint { x: 3, y: -2, z: 5 }),
                CellPoint { x: 3, y: -2, z: 5 }
            );
        }
    }

    #[test]
    fn every_rotation_carries_the_front_to_its_own_direction() {
        for &facing in &ALL {
            assert_eq!(facing.rotate_direction(CellFacing::PosZ), facing);
        }
    }

    #[test]
    fn unrotate_inverts_rotate_for_every_pair() {
        for &facing in &ALL {
            for &direction in &ALL {
                assert_eq!(
                    facing.unrotate_direction(facing.rotate_direction(direction)),
                    direction
                );
                assert_eq!(
                    facing.rotate_direction(facing.unrotate_direction(direction)),
                    direction
                );
            }
        }
    }

    #[test]
    fn rotated_directions_are_also_facings() {
        for &facing in &ALL {
            for &direction in &ALL {
                let rotated = facing.rotate_direction(direction);
                assert!(ALL.contains(&rotated));
                assert_eq!(rotated.unit_vector(), {
                    let [x, y, z] = direction.unit_vector();
                    facing.remap_vector(x, y, z)
                });
            }
        }
    }

    #[test]
    fn point_remap_pins_the_cardinal_axis_tables() {
        let point = CellPoint { x: 1, y: 2, z: 3 };
        assert_eq!(
            CellFacing::PosX.remap_point(point),
            CellPoint { x: 3, y: 2, z: -1 }
        );
        assert_eq!(
            CellFacing::NegX.remap_point(point),
            CellPoint { x: -3, y: 2, z: 1 }
        );
        assert_eq!(
            CellFacing::PosY.remap_point(point),
            CellPoint { x: 1, y: 3, z: -2 }
        );
        assert_eq!(
            CellFacing::NegY.remap_point(point),
            CellPoint { x: 1, y: -3, z: 2 }
        );
        assert_eq!(CellFacing::PosZ.remap_point(point), point);
        assert_eq!(
            CellFacing::NegZ.remap_point(point),
            CellPoint { x: -1, y: 2, z: -3 }
        );
    }

    #[test]
    fn global_direction_mapping_roundtrips() {
        for &facing in &ALL {
            assert_eq!(
                CellFacing::from_global_direction(facing.to_global_direction()),
                facing
            );
        }
        assert_eq!(
            CellFacing::PosZ.to_global_direction(),
            GlobalDirection::South
        );
        assert_eq!(
            CellFacing::NegZ.to_global_direction(),
            GlobalDirection::North
        );
        assert_eq!(
            CellFacing::PosX.to_global_direction(),
            GlobalDirection::East
        );
        assert_eq!(CellFacing::PosY.to_global_direction(), GlobalDirection::Top);
    }

    #[test]
    fn relative_facing_with_identity_layers_is_the_cell_facing() {
        for &cell in &ALL {
            assert_eq!(
                CellFacing::relative_facing(cell, CellFacing::PosZ, CellFacing::PosZ),
                cell
            );
        }
    }

    const ROTATIONS_24: [FacingRotation; 24] = {
        let mut rotations = [FacingRotation::IDENTITY; 24];
        let mut index = 0;
        let mut face_index = 0;
        while face_index < ALL_FACING_CONSTS.len() {
            let mut roll_index = 0;
            while roll_index < 4 {
                rotations[index] = FacingRotation::new(
                    ALL_FACING_CONSTS[face_index],
                    [
                        CellRoll::Deg0,
                        CellRoll::Deg90,
                        CellRoll::Deg180,
                        CellRoll::Deg270,
                    ][roll_index],
                );
                index += 1;
                roll_index += 1;
            }
            face_index += 1;
        }
        rotations
    };

    const ALL_FACING_CONSTS: [CellFacing; 6] = [
        CellFacing::PosX,
        CellFacing::NegX,
        CellFacing::PosY,
        CellFacing::NegY,
        CellFacing::PosZ,
        CellFacing::NegZ,
    ];

    #[test]
    fn the_24_rotations_are_distinct() {
        for (i, &a) in ROTATIONS_24.iter().enumerate() {
            for &b in &ROTATIONS_24[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn every_rotation_preserves_the_front_as_its_facing_component() {
        for &rotation in &ROTATIONS_24 {
            assert_eq!(rotation.rotate_direction(CellFacing::PosZ), rotation.facing);
        }
    }

    #[test]
    fn compose_and_inverse_obey_the_group_laws_over_all_24_elements() {
        for &a in &ROTATIONS_24 {
            for &b in &ROTATIONS_24 {
                // Closure: composition stays inside the 24.
                let ab = a.compose(b);
                assert!(ROTATIONS_24.contains(&ab));
                // Inverses.
                assert_eq!(a.compose(a.inverse()), FacingRotation::IDENTITY);
                assert_eq!(a.inverse().compose(a), FacingRotation::IDENTITY);
                for &c in &ROTATIONS_24 {
                    // Associativity over all 24-cubed triples.
                    assert_eq!(a.compose(b).compose(c), a.compose(b.compose(c)));
                }
            }
        }
    }

    #[test]
    fn deg0_roll_rotation_matches_the_plain_facing_action() {
        for &facing in &ALL {
            let rotation = FacingRotation::from_facing(facing);
            for &direction in &ALL {
                assert_eq!(
                    rotation.rotate_direction(direction),
                    facing.rotate_direction(direction)
                );
            }
        }
    }

    #[test]
    fn roll_rotates_lateral_directions_and_keeps_the_front() {
        assert_eq!(
            FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90)
                .rotate_direction(CellFacing::PosX),
            CellFacing::PosY
        );
        assert_eq!(
            FacingRotation::new(CellFacing::PosX, CellRoll::Deg90)
                .rotate_direction(CellFacing::PosZ),
            CellFacing::PosX
        );
    }

    #[test]
    fn camera_roll_tags_the_art_without_remapping_the_side() {
        // A rolled camera is an image rotation: for a fixed swing, the roll
        // must never change which side a cell resolves — only the roll tag.
        // A stick's end grain (front facing PosY) stays on its own pos-y
        // key at every camera roll instead of stealing a rolled body side's
        // lying-profile key.
        for &camera_facing in &ALL {
            let baseline = FacingRotation::relative_rotation(
                FacingRotation::from_facing(CellFacing::PosY),
                FacingRotation::IDENTITY,
                FacingRotation::new(camera_facing, CellRoll::Deg0),
            );
            for &camera_roll in &CellRoll::ALL {
                let relative = FacingRotation::relative_rotation(
                    FacingRotation::from_facing(CellFacing::PosY),
                    FacingRotation::IDENTITY,
                    FacingRotation::new(camera_facing, camera_roll),
                );
                assert_eq!(
                    relative.facing, baseline.facing,
                    "camera roll must not remap the side direction"
                );
            }
        }
    }

    #[test]
    fn relative_rotation_facing_component_matches_relative_facing() {
        for &cell in &ALL {
            for &group in &ALL {
                for &camera in &ALL {
                    let rotation = FacingRotation::relative_rotation(
                        FacingRotation::from_facing(cell),
                        FacingRotation::from_facing(group),
                        FacingRotation::from_facing(camera),
                    );
                    assert_eq!(
                        rotation.facing,
                        CellFacing::relative_facing(cell, group, camera)
                    );
                }
            }
        }
    }

    #[test]
    fn facing_only_composition_pins_the_lateral_state_the_quotient_hides() {
        // Two no-roll rotations can compose to a rolled rotation: the full
        // group keeps the lateral orientation the facing-only resolution
        // discarded. A PosX-oriented cell expressed in the PosY-viewing
        // camera's frame is one such case.
        let relative = FacingRotation::relative_rotation(
            FacingRotation::from_facing(CellFacing::PosX),
            FacingRotation::IDENTITY,
            FacingRotation::from_facing(CellFacing::PosY),
        );
        assert_eq!(
            relative,
            FacingRotation::new(CellFacing::PosX, CellRoll::Deg90)
        );
    }

    #[test]
    fn camera_roll_shows_up_in_the_relative_roll_component() {
        // Default camera (views from North), cell fronting North on an
        // identity group: front view at every camera roll. The camera roll
        // is an image rotation, so it lands in the relative roll component
        // one to one and never touches the facing component.
        for &camera_roll in &CellRoll::ALL {
            let relative = FacingRotation::relative_rotation(
                FacingRotation::from_facing(CellFacing::NegZ),
                FacingRotation::IDENTITY,
                FacingRotation::new(CellFacing::NegZ, camera_roll),
            );
            assert_eq!(relative.facing, CellFacing::PosZ);
            assert_eq!(relative.roll, camera_roll);
        }
    }

    #[test]
    fn group_facing_rotates_the_cell_front() {
        // A cell facing PosZ on a group facing PosX fronts along PosX.
        assert_eq!(
            CellFacing::relative_facing(CellFacing::PosZ, CellFacing::PosX, CellFacing::PosZ),
            CellFacing::PosX
        );
        // A cell facing PosX on a group facing PosX fronts along NegZ (the
        // PosX rotation maps +z to +x and +x to -z).
        assert_eq!(
            CellFacing::relative_facing(CellFacing::PosX, CellFacing::PosX, CellFacing::PosZ),
            CellFacing::NegZ
        );
    }

    #[test]
    fn camera_frame_expresses_the_front_relative_to_the_view() {
        // Front toward camera-frame +z when the camera rotation cancels the
        // world front direction.
        assert_eq!(
            CellFacing::relative_facing(CellFacing::NegZ, CellFacing::PosZ, CellFacing::NegZ),
            CellFacing::PosZ
        );
        // Camera and group rotating together leaves the relative front alone.
        for &cell in &ALL {
            assert_eq!(
                CellFacing::relative_facing(cell, CellFacing::PosX, CellFacing::PosX),
                CellFacing::relative_facing(cell, CellFacing::PosZ, CellFacing::PosZ),
            );
        }
    }
}
