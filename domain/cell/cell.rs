use crate::{
    cell_materials::CellMaterialId, coordinate_space::CellPoint, CellColor, CellFacing,
    CellGraphic, CellTexture, CellWarble, CellWeight,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub position: CellPoint,
    pub graphic: CellGraphic,
    pub color: CellColor,
    pub weight: CellWeight,
    pub texture: CellTexture,
    pub warble: CellWarble,
    /// The cell's own orientation over the six cardinals. Composed additively
    /// with the group and camera facings into one relative facing at render
    /// time; the default leaves the cell facing-invariant.
    pub facing: CellFacing,
    pub shader_stack: Vec<u32>,
}

impl Cell {
    pub fn is_visible(&self) -> bool {
        self.graphic.is_visible()
    }

    /// Applies one won side graphic onto this cell: only the cell's graphic
    /// is replaced. Color, weight, and shader stack are cell-level state —
    /// the cell's color slots decide what materials are, so a side never
    /// overrides them.
    pub fn apply_side_graphic(mut self, graphic: &crate::cell_graphic::SideGraphic) -> Self {
        self.graphic = match graphic {
            crate::cell_graphic::SideGraphic::None => CellGraphic::None,
            crate::cell_graphic::SideGraphic::Glyph(glyph) => CellGraphic::Glyph(*glyph),
            crate::cell_graphic::SideGraphic::Sprite(sprite) => CellGraphic::Sprite(sprite.clone()),
        };
        self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            position: CellPoint::origin(),
            graphic: CellGraphic::default(),
            color: CellColor::default(),
            weight: CellWeight::default(),
            texture: CellTexture::default(),
            warble: CellWarble::default(),
            facing: CellFacing::default(),
            shader_stack: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_side_graphic_replaces_only_the_graphic() {
        let cell = Cell {
            position: CellPoint { x: 2, y: 3, z: 0 },
            graphic: CellGraphic::Sprite(crate::cell_graphic::SpriteGraphic::new(
                "proofs/chest.png",
            )),
            color: CellColor::Flat([1.0, 0.0, 0.0, 1.0]),
            weight: CellWeight::Three,
            shader_stack: vec![crate::CELL_SHADER_WEIGHT_SIN],
            ..Cell::default()
        }
        .apply_side_graphic(&crate::cell_graphic::SideGraphic::Glyph('F'));

        // Position and every cell-level appearance channel survive; only the
        // graphic comes from the side.
        assert_eq!(cell.position, CellPoint { x: 2, y: 3, z: 0 });
        assert_eq!(cell.graphic, CellGraphic::Glyph('F'));
        assert_eq!(cell.color, CellColor::Flat([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(cell.weight, CellWeight::Three);
        assert_eq!(cell.shader_stack, vec![crate::CELL_SHADER_WEIGHT_SIN]);
    }

    #[test]
    fn apply_side_graphic_none_hides_the_cell() {
        let cell = Cell::default().apply_side_graphic(&crate::cell_graphic::SideGraphic::None);

        assert_eq!(cell.graphic, CellGraphic::None);
    }

    #[test]
    fn default_cell_is_local_and_uses_the_canonical_slot_defaults() {
        let cell = Cell::default();

        assert_eq!(cell.position, CellPoint::origin());
        assert_eq!(cell.graphic, CellGraphic::None);
        assert_eq!(
            cell.color,
            CellColor::Material(CellMaterialId::GrayScale),
            "default color shades real bands instead of a flat white silhouette"
        );
        assert_eq!(cell.weight, CellWeight::Zero);
        assert_eq!(cell.texture, CellTexture::none());
        assert_eq!(cell.warble, CellWarble::none());
        assert_eq!(cell.facing, CellFacing::PosZ);
        assert!(cell.shader_stack.is_empty());
    }

    #[test]
    fn cell_visibility_is_owned_by_the_graphic_slot() {
        assert!(!Cell::default().is_visible());
        assert!(Cell {
            graphic: CellGraphic::Glyph('█'),
            ..Cell::default()
        }
        .is_visible());
    }
}
