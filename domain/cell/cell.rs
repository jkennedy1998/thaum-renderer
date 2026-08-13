use crate::{
    coordinate_space::CellPoint, CellColor, CellGraphic, CellTexture, CellWarble, CellWeight,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub position: CellPoint,
    pub graphic: CellGraphic,
    pub color: CellColor,
    pub weight: CellWeight,
    pub texture: CellTexture,
    pub warble: CellWarble,
    pub shader_stack: Vec<u32>,
}

impl Cell {
    pub fn is_visible(&self) -> bool {
        self.graphic.is_visible()
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
            shader_stack: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cell_is_local_and_uses_the_canonical_slot_defaults() {
        let cell = Cell::default();

        assert_eq!(cell.position, CellPoint::origin());
        assert_eq!(cell.graphic, CellGraphic::None);
        assert_eq!(cell.color, CellColor::Flat([1.0, 1.0, 1.0, 1.0]));
        assert_eq!(cell.weight, CellWeight::Zero);
        assert_eq!(cell.texture, CellTexture::none());
        assert_eq!(cell.warble, CellWarble::none());
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
