use std::collections::BTreeMap;

use crate::{Cell, CellPoint, WorldPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellClip {
    pub min: CellPoint,
    pub max: CellPoint,
}

impl CellClip {
    pub fn contains(self, point: CellPoint) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellBounds {
    pub min: CellPoint,
    pub max: CellPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CellGroupFacing {
    PosX,
    NegX,
    PosY,
    NegY,
    #[default]
    PosZ,
    NegZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CellGroupIntakeBehavior {
    #[default]
    Rotating3d,
    Flat2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellGroup {
    pub origin: WorldPoint,
    pub facing: CellGroupFacing,
    pub intake_behavior: CellGroupIntakeBehavior,
    pub cells: BTreeMap<CellPoint, Cell>,
    pub clip: Option<CellClip>,
}

impl CellGroup {
    pub fn new(origin: WorldPoint) -> Self {
        Self {
            origin,
            facing: CellGroupFacing::default(),
            intake_behavior: CellGroupIntakeBehavior::default(),
            cells: BTreeMap::new(),
            clip: None,
        }
    }

    pub fn from_cells(origin: WorldPoint, cells: impl IntoIterator<Item = Cell>) -> Self {
        let mut group = Self::new(origin);
        group.extend(cells);
        group
    }

    pub fn with_clip(mut self, clip: CellClip) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn with_facing(mut self, facing: CellGroupFacing) -> Self {
        self.facing = facing;
        self
    }

    pub fn with_intake_behavior(mut self, intake_behavior: CellGroupIntakeBehavior) -> Self {
        self.intake_behavior = intake_behavior;
        self
    }

    pub fn insert(&mut self, cell: Cell) -> Option<Cell> {
        self.cells.insert(cell.position, cell)
    }

    pub fn extend(&mut self, cells: impl IntoIterator<Item = Cell>) {
        for cell in cells {
            self.insert(cell);
        }
    }

    pub fn get(&self, position: CellPoint) -> Option<&Cell> {
        self.cells.get(&position)
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells
            .values()
            .filter(|cell| self.cell_is_visible(cell))
    }

    pub fn bounds(&self) -> Option<CellBounds> {
        let mut cells = self.cells.values();
        let first = cells.next()?;
        let mut min = first.position;
        let mut max = first.position;

        for cell in cells {
            min.x = min.x.min(cell.position.x);
            min.y = min.y.min(cell.position.y);
            min.z = min.z.min(cell.position.z);
            max.x = max.x.max(cell.position.x);
            max.y = max.y.max(cell.position.y);
            max.z = max.z.max(cell.position.z);
        }

        Some(CellBounds { min, max })
    }

    pub fn transformed_local_point(&self, local: CellPoint) -> CellPoint {
        match self.facing {
            CellGroupFacing::PosX => CellPoint {
                x: local.z,
                y: local.y,
                z: -local.x,
            },
            CellGroupFacing::NegX => CellPoint {
                x: -local.z,
                y: local.y,
                z: local.x,
            },
            CellGroupFacing::PosY => CellPoint {
                x: local.x,
                y: local.z,
                z: -local.y,
            },
            CellGroupFacing::NegY => CellPoint {
                x: local.x,
                y: -local.z,
                z: local.y,
            },
            CellGroupFacing::PosZ => local,
            CellGroupFacing::NegZ => CellPoint {
                x: -local.x,
                y: local.y,
                z: -local.z,
            },
        }
    }

    pub fn world_point_for(&self, local: CellPoint) -> WorldPoint {
        let local = self.transformed_local_point(local);

        WorldPoint {
            x: self.origin.x + local.x,
            y: self.origin.y + local.y,
            z: self.origin.z + local.z,
        }
    }

    fn cell_is_visible(&self, cell: &Cell) -> bool {
        self.clip
            .map(|clip| clip.contains(cell.position))
            .unwrap_or(true)
    }
}

impl Default for CellGroup {
    fn default() -> Self {
        Self::new(WorldPoint::origin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellColor, CellGraphic, CellTexture, CellWarble, CellWeight};

    #[test]
    fn sparse_storage_replaces_existing_cell_at_same_local_position() {
        let mut group = CellGroup::new(WorldPoint::origin());
        group.insert(Cell {
            position: CellPoint { x: 2, y: -1, z: 3 },
            graphic: CellGraphic::Glyph('A'),
            ..Cell::default()
        });
        group.insert(Cell {
            position: CellPoint { x: 2, y: -1, z: 3 },
            graphic: CellGraphic::Glyph('B'),
            color: CellColor::Flat([0.0, 1.0, 0.0, 1.0]),
            ..Cell::default()
        });

        assert_eq!(group.cells.len(), 1);
        assert_eq!(
            group.get(CellPoint { x: 2, y: -1, z: 3 }).unwrap().graphic,
            CellGraphic::Glyph('B')
        );
    }

    #[test]
    fn bounds_cover_the_inserted_local_cells() {
        let group = CellGroup::from_cells(
            WorldPoint::origin(),
            [
                Cell {
                    position: CellPoint { x: -2, y: 4, z: 1 },
                    ..Cell::default()
                },
                Cell {
                    position: CellPoint { x: 3, y: -1, z: 5 },
                    ..Cell::default()
                },
            ],
        );

        assert_eq!(
            group.bounds(),
            Some(CellBounds {
                min: CellPoint { x: -2, y: -1, z: 1 },
                max: CellPoint { x: 3, y: 4, z: 5 },
            })
        );
    }

    #[test]
    fn iter_cells_applies_local_clipping_before_composition() {
        let group = CellGroup::from_cells(
            WorldPoint::origin(),
            [
                Cell {
                    position: CellPoint { x: 0, y: 0, z: 0 },
                    graphic: CellGraphic::Glyph('A'),
                    ..Cell::default()
                },
                Cell {
                    position: CellPoint { x: 3, y: 0, z: 0 },
                    graphic: CellGraphic::Glyph('B'),
                    ..Cell::default()
                },
            ],
        )
        .with_clip(CellClip {
            min: CellPoint { x: 0, y: 0, z: 0 },
            max: CellPoint { x: 1, y: 0, z: 0 },
        });

        let visible = group.iter_cells().collect::<Vec<_>>();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].graphic, CellGraphic::Glyph('A'));
    }

    #[test]
    fn world_point_for_translates_local_points_by_group_origin() {
        let group = CellGroup::new(WorldPoint { x: 7, y: -3, z: 2 });

        assert_eq!(
            group.world_point_for(CellPoint { x: 1, y: -4, z: 5 }),
            WorldPoint { x: 8, y: -7, z: 7 }
        );
    }

    #[test]
    fn six_cardinal_facing_reorients_the_local_forward_axis() {
        let cases = [
            (CellGroupFacing::PosX, CellPoint { x: 1, y: 0, z: 0 }),
            (CellGroupFacing::NegX, CellPoint { x: -1, y: 0, z: 0 }),
            (CellGroupFacing::PosY, CellPoint { x: 0, y: 1, z: 0 }),
            (CellGroupFacing::NegY, CellPoint { x: 0, y: -1, z: 0 }),
            (CellGroupFacing::PosZ, CellPoint { x: 0, y: 0, z: 1 }),
            (CellGroupFacing::NegZ, CellPoint { x: 0, y: 0, z: -1 }),
        ];

        for (facing, expected) in cases {
            let group = CellGroup::new(WorldPoint::origin()).with_facing(facing);
            assert_eq!(
                group.transformed_local_point(CellPoint { x: 0, y: 0, z: 1 }),
                expected
            );
        }
    }

    #[test]
    fn insert_preserves_full_cell_slot_data() {
        let mut group = CellGroup::new(WorldPoint::origin());
        group.insert(Cell {
            position: CellPoint { x: 1, y: 2, z: 3 },
            graphic: CellGraphic::Glyph('*'),
            color: CellColor::Flat([0.5, 0.6, 0.7, 1.0]),
            weight: CellWeight::Three,
            texture: CellTexture::new(11),
            warble: CellWarble::new(12),
            shader_stack: vec![9, 10],
        });

        assert_eq!(
            group.get(CellPoint { x: 1, y: 2, z: 3 }),
            Some(&Cell {
                position: CellPoint { x: 1, y: 2, z: 3 },
                graphic: CellGraphic::Glyph('*'),
                color: CellColor::Flat([0.5, 0.6, 0.7, 1.0]),
                weight: CellWeight::Three,
                texture: CellTexture::new(11),
                warble: CellWarble::new(12),
                shader_stack: vec![9, 10],
            })
        );
    }

    #[test]
    fn intake_behavior_defaults_to_rotating_3d() {
        let group = CellGroup::new(WorldPoint::origin());
        assert_eq!(group.intake_behavior, CellGroupIntakeBehavior::Rotating3d);
    }

    #[test]
    fn intake_behavior_can_be_switched_to_flat_2d() {
        let group = CellGroup::new(WorldPoint::origin())
            .with_intake_behavior(CellGroupIntakeBehavior::Flat2d);
        assert_eq!(group.intake_behavior, CellGroupIntakeBehavior::Flat2d);
    }
}
