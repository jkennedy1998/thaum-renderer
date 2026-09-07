use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{Cell, CellColor, Composition};

/// Fast content identity for a composition: a within-process-stable hash of
/// everything the scene build reads from it (group origins, facing, intake,
/// clip, every cell's full slot payload, pass order, HUD opt-in). Boot's
/// scene cache uses it as the composition half of its fingerprint, so a
/// frame whose composition came out identical reuses the previous scene
/// without re-projecting quads or rebuilding the glyph atlas.
///
/// `CellColor`'s float payloads hash through `to_bits` so equal bits hash
/// equal; NaN payload equality follows bit equality, matching how colors
/// flow (they are written once, never computed into NaN variants).
pub fn composition_content_hash(composition: &Composition) -> u64 {
    let mut hasher = DefaultHasher::new();
    composition.flat_2d_screen_locked.hash(&mut hasher);
    composition.pass_order.hash(&mut hasher);
    for group in &composition.groups {
        group.origin.hash(&mut hasher);
        group.facing.hash(&mut hasher);
        group.intake_behavior.hash(&mut hasher);
        group.clip.hash(&mut hasher);
        // BTreeMap iteration order is deterministic, so sequential hashing
        // yields the same digest for equal group content.
        for (point, cell) in &group.cells {
            point.hash(&mut hasher);
            hash_cell(cell, &mut hasher);
        }
    }
    hasher.finish()
}

fn hash_cell(cell: &Cell, hasher: &mut DefaultHasher) {
    hash_graphic(&cell.graphic, hasher);
    hash_color(&cell.color, hasher);
    cell.weight.hash(hasher);
    cell.texture.code().hash(hasher);
    cell.warble.code().hash(hasher);
    cell.shader_stack.hash(hasher);
}

fn hash_graphic(graphic: &crate::CellGraphic, hasher: &mut DefaultHasher) {
    use crate::CellGraphic;
    std::mem::discriminant(graphic).hash(hasher);
    match graphic {
        CellGraphic::None => {}
        CellGraphic::Glyph(glyph) => glyph.hash(hasher),
        CellGraphic::Sprite(sprite) => sprite.atlas_relative_path().as_os_str().hash(hasher),
    }
}

fn hash_color(color: &CellColor, hasher: &mut DefaultHasher) {
    std::mem::discriminant(color).hash(hasher);
    match color {
        CellColor::Flat(rgba) => rgba.map(|channel| channel.to_bits()).hash(hasher),
        CellColor::Material(material) => material.hash(hasher),
        CellColor::Slots { a, b, c } => {
            for slot in [a, b, c] {
                std::mem::discriminant(slot).hash(hasher);
                match slot {
                    crate::CellColorSlot::Flat(rgba) => {
                        rgba.map(|channel| channel.to_bits()).hash(hasher)
                    }
                    crate::CellColorSlot::Material(material) => material.hash(hasher),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellGraphic, CellGroup, CellPoint, CellWeight, WorldPoint};

    fn composition_with_cell(cell: Cell) -> Composition {
        let mut group = CellGroup::new(WorldPoint::origin());
        group.insert(Cell {
            position: CellPoint { x: 1, y: 2, z: 3 },
            ..cell
        });
        Composition::ordered(vec![group])
    }

    #[test]
    fn identical_compositions_hash_equal() {
        let build = || {
            let mut group = CellGroup::new(WorldPoint { x: 4, y: 5, z: 6 });
            group.insert(Cell {
                position: CellPoint { x: 1, y: 1, z: 0 },
                graphic: CellGraphic::Glyph('█'),
                weight: CellWeight::Two,
                ..Cell::default()
            });
            Composition::ordered(vec![group])
        };

        assert_eq!(
            composition_content_hash(&build()),
            composition_content_hash(&build())
        );
    }

    #[test]
    fn cell_slot_changes_change_the_hash() {
        let base = composition_with_cell(Cell {
            graphic: CellGraphic::Glyph('A'),
            ..Cell::default()
        });
        let mut other = base.clone();
        other.groups[0]
            .cells
            .get_mut(&CellPoint { x: 1, y: 2, z: 3 })
            .unwrap()
            .weight = CellWeight::Three;
        assert_ne!(
            composition_content_hash(&base),
            composition_content_hash(&other)
        );
    }

    #[test]
    fn cell_removal_changes_the_hash() {
        let mut group = CellGroup::new(WorldPoint::origin());
        group.insert(Cell::default());
        let base = Composition::ordered(vec![group]);

        let mut other = base.clone();
        other.groups[0].cells.remove(&CellPoint::origin());

        assert_ne!(
            composition_content_hash(&base),
            composition_content_hash(&other)
        );
    }

    #[test]
    fn pass_order_is_part_of_the_identity() {
        let mut group_a = CellGroup::new(WorldPoint::origin());
        group_a.insert(Cell::default());
        let mut group_b = CellGroup::new(WorldPoint { x: 9, y: 0, z: 0 });
        group_b.insert(Cell::default());
        let base = Composition::ordered(vec![group_a.clone(), group_b.clone()]);
        let reordered = base.clone().with_pass_order(vec![1, 0]);

        assert_ne!(
            composition_content_hash(&base),
            composition_content_hash(&reordered)
        );
    }
}
