use std::collections::HashMap;

use crate::{Cell, CellGroup, CellGroupIntakeBehavior, WorldPoint};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Composition {
    pub groups: Vec<CellGroup>,
    pub pass_order: Vec<usize>,
    /// Producer-maintained content generation. `0` means unset: boot's scene
    /// cache then falls back to hashing the composition content per frame.
    /// A producer that rebuilds the composition only when its inputs change
    /// should bump this on every rebuild, making the cache's per-frame
    /// identity check O(1). Non-zero values are trusted as content identity.
    pub revision: u64,
    /// Opt-in screen-locked 2D layer. When false (default), `Flat2d` groups
    /// are world-anchored: the group origin is a world point, so Flat2d
    /// content drifts naturally with camera pans and swings. When true,
    /// `Flat2d` groups are a screen-locked HUD layer: the origin is a
    /// camera-unit offset from the camera's focus target and only
    /// `hud_pan_offset` can move it — the behavior painter HUD panels want.
    /// Programs opt in per composition; the shared scene stays neutral.
    pub flat_2d_screen_locked: bool,
}

impl Composition {
    pub fn ordered(groups: Vec<CellGroup>) -> Self {
        Self {
            groups,
            pass_order: Vec::new(),
            revision: 0,
            flat_2d_screen_locked: false,
        }
        .with_natural_pass_order()
    }

    /// Sets the producer-maintained content generation (see the field docs).
    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// Opts this composition's Flat2d layer into screen-locked HUD behavior
    /// (origin as camera-unit offset from the focus target, movable only by
    /// `hud_pan_offset`). See the field docs for the two modes.
    pub fn with_flat_2d_screen_locked(mut self, locked: bool) -> Self {
        self.flat_2d_screen_locked = locked;
        self
    }

    pub fn with_natural_pass_order(mut self) -> Self {
        self.pass_order = (0..self.groups.len()).collect();
        self
    }

    pub fn with_pass_order(mut self, pass_order: Vec<usize>) -> Self {
        self.pass_order = pass_order;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComposedCell {
    pub world: WorldPoint,
    pub group_origin: WorldPoint,
    pub intake_behavior: CellGroupIntakeBehavior,
    pub cell: Cell,
}

pub fn compose_cells(composition: &Composition) -> Vec<ComposedCell> {
    let mut composed = Vec::new();
    let mut world_to_index = HashMap::<(i32, i32, i32), usize>::new();

    for group_index in composition.pass_order.iter().copied() {
        let group = composition.groups.get(group_index).unwrap_or_else(|| {
            panic!("composition pass_order references missing group index {group_index}")
        });

        for cell in group.iter_cells() {
            let world = group.world_point_for(cell.position);
            let key = (world.x, world.y, world.z);

            if let Some(index) = world_to_index.get(&key).copied() {
                composed[index] = ComposedCell {
                    world,
                    group_origin: group.origin,
                    intake_behavior: group.intake_behavior,
                    cell: cell.clone(),
                };
                continue;
            }

            let index = composed.len();
            composed.push(ComposedCell {
                world,
                group_origin: group.origin,
                intake_behavior: group.intake_behavior,
                cell: cell.clone(),
            });
            world_to_index.insert(key, index);
        }
    }

    composed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellColor, CellGraphic, CellGroupFacing, CellGroupIntakeBehavior, CellPoint};

    #[test]
    fn compose_cells_keeps_non_overlapping_world_cells() {
        let composition = Composition::ordered(vec![
            CellGroup::from_cells(
                WorldPoint::origin(),
                [Cell {
                    position: CellPoint::origin(),
                    graphic: CellGraphic::Glyph('A'),
                    color: CellColor::Flat([1.0, 0.0, 0.0, 1.0]),
                    ..Cell::default()
                }],
            ),
            CellGroup::from_cells(
                WorldPoint { x: 1, y: 0, z: 1 },
                [Cell {
                    position: CellPoint::origin(),
                    graphic: CellGraphic::Glyph('B'),
                    color: CellColor::Flat([0.0, 1.0, 0.0, 1.0]),
                    ..Cell::default()
                }],
            ),
        ]);

        let composed = compose_cells(&composition);
        assert_eq!(composed.len(), 2);
        assert_eq!(composed[0].world, WorldPoint::origin());
        assert_eq!(composed[0].group_origin, WorldPoint::origin());
        assert_eq!(
            composed[0].intake_behavior,
            CellGroupIntakeBehavior::Rotating3d
        );
        assert_eq!(composed[1].world, WorldPoint { x: 1, y: 0, z: 1 });
    }

    #[test]
    fn compose_cells_replaces_earlier_cells_at_exact_same_world_xyz() {
        let composition = Composition::ordered(vec![
            CellGroup::from_cells(
                WorldPoint { x: 3, y: -2, z: 1 },
                [Cell {
                    position: CellPoint::origin(),
                    graphic: CellGraphic::Glyph('A'),
                    color: CellColor::Flat([1.0, 0.0, 0.0, 1.0]),
                    ..Cell::default()
                }],
            ),
            CellGroup::from_cells(
                WorldPoint { x: 2, y: -2, z: 1 },
                [Cell {
                    position: CellPoint { x: 1, y: 0, z: 0 },
                    graphic: CellGraphic::Glyph('B'),
                    color: CellColor::Flat([0.0, 1.0, 0.0, 1.0]),
                    ..Cell::default()
                }],
            ),
        ]);

        let composed = compose_cells(&composition);
        assert_eq!(composed.len(), 1);
        assert_eq!(composed[0].world, WorldPoint { x: 3, y: -2, z: 1 });
        assert_eq!(composed[0].group_origin, WorldPoint { x: 2, y: -2, z: 1 });
        assert_eq!(
            composed[0].intake_behavior,
            CellGroupIntakeBehavior::Rotating3d
        );
        assert_eq!(composed[0].cell.graphic, CellGraphic::Glyph('B'));
        assert_eq!(
            composed[0].cell.color,
            CellColor::Flat([0.0, 1.0, 0.0, 1.0])
        );
    }

    #[test]
    fn compose_cells_uses_explicit_pass_order_instead_of_group_vector_order() {
        let composition = Composition {
            groups: vec![
                CellGroup::from_cells(
                    WorldPoint { x: 2, y: 1, z: 0 },
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 0.0, 0.0, 1.0]),
                        ..Cell::default()
                    }],
                ),
                CellGroup::from_cells(
                    WorldPoint { x: 1, y: 1, z: 0 },
                    [Cell {
                        position: CellPoint { x: 1, y: 0, z: 0 },
                        graphic: CellGraphic::Glyph('B'),
                        color: CellColor::Flat([0.0, 1.0, 0.0, 1.0]),
                        ..Cell::default()
                    }],
                ),
            ],
            pass_order: vec![1, 0],
            revision: 0,
            flat_2d_screen_locked: false,
        };

        let composed = compose_cells(&composition);
        assert_eq!(composed.len(), 1);
        assert_eq!(composed[0].world, WorldPoint { x: 2, y: 1, z: 0 });
        assert_eq!(composed[0].group_origin, WorldPoint { x: 2, y: 1, z: 0 });
        assert_eq!(
            composed[0].intake_behavior,
            CellGroupIntakeBehavior::Rotating3d
        );
        assert_eq!(composed[0].cell.graphic, CellGraphic::Glyph('A'));
        assert_eq!(
            composed[0].cell.color,
            CellColor::Flat([1.0, 0.0, 0.0, 1.0])
        );
    }

    #[test]
    fn compose_cells_respects_group_facing_when_mapping_local_to_world() {
        let composition = Composition::ordered(vec![CellGroup::from_cells(
            WorldPoint { x: 4, y: -2, z: 1 },
            [Cell {
                position: CellPoint { x: 0, y: 0, z: 1 },
                graphic: CellGraphic::Glyph('F'),
                ..Cell::default()
            }],
        )
        .with_facing(CellGroupFacing::PosX)]);

        let composed = compose_cells(&composition);
        assert_eq!(composed.len(), 1);
        assert_eq!(composed[0].world, WorldPoint { x: 5, y: -2, z: 1 });
        assert_eq!(composed[0].cell.graphic, CellGraphic::Glyph('F'));
    }

    #[test]
    fn compose_cells_keeps_group_intake_behavior() {
        let composition = Composition::ordered(vec![CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('2'),
                ..Cell::default()
            }],
        )
        .with_intake_behavior(CellGroupIntakeBehavior::Flat2d)]);

        let composed = compose_cells(&composition);
        assert_eq!(composed[0].intake_behavior, CellGroupIntakeBehavior::Flat2d);
        assert_eq!(composed[0].group_origin, WorldPoint::origin());
    }
}
