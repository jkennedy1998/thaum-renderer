use crate::{cell_materials::ColorBand, CellMaterialId, SpriteColorChannel};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellColor {
    Flat([f32; 4]),
    Material(CellMaterialId),
    Slots {
        a: CellColorSlot,
        b: CellColorSlot,
        c: CellColorSlot,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellColorSlot {
    Flat([f32; 4]),
    Material(CellMaterialId),
}

impl Default for CellColor {
    fn default() -> Self {
        Self::Flat([1.0, 1.0, 1.0, 1.0])
    }
}

impl CellColor {
    pub const fn slots(a: CellColorSlot, b: CellColorSlot, c: CellColorSlot) -> Self {
        Self::Slots { a, b, c }
    }

    pub fn resolve_glyph(self) -> [f32; 4] {
        self.resolve_sprite(SpriteColorChannel::A, ColorBand::MediumLight)
    }

    pub fn resolve_sprite(self, channel: SpriteColorChannel, band: ColorBand) -> [f32; 4] {
        match channel {
            SpriteColorChannel::A => self.resolve_primary_slot(SpritePrimarySlot::A, band),
            SpriteColorChannel::B => self.resolve_primary_slot(SpritePrimarySlot::B, band),
            SpriteColorChannel::C => self.resolve_primary_slot(SpritePrimarySlot::C, band),
            SpriteColorChannel::AB => mix_colors(
                self.resolve_primary_slot(SpritePrimarySlot::A, band),
                self.resolve_primary_slot(SpritePrimarySlot::B, band),
            ),
            SpriteColorChannel::BC => mix_colors(
                self.resolve_primary_slot(SpritePrimarySlot::B, band),
                self.resolve_primary_slot(SpritePrimarySlot::C, band),
            ),
            SpriteColorChannel::CA => mix_colors(
                self.resolve_primary_slot(SpritePrimarySlot::C, band),
                self.resolve_primary_slot(SpritePrimarySlot::A, band),
            ),
        }
    }

    fn resolve_primary_slot(self, slot: SpritePrimarySlot, band: ColorBand) -> [f32; 4] {
        match self.slot_assignment(slot) {
            CellColorSlot::Flat(color) => color,
            CellColorSlot::Material(material) => material.resolve_band(band),
        }
    }

    const fn slot_assignment(self, slot: SpritePrimarySlot) -> CellColorSlot {
        match self {
            Self::Flat(color) => CellColorSlot::Flat(color),
            Self::Material(material) => CellColorSlot::Material(material),
            Self::Slots { a, b, c } => match slot {
                SpritePrimarySlot::A => a,
                SpritePrimarySlot::B => b,
                SpritePrimarySlot::C => c,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpritePrimarySlot {
    A,
    B,
    C,
}

fn mix_colors(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        (left[0] + right[0]) * 0.5,
        (left[1] + right[1]) * 0.5,
        (left[2] + right[2]) * 0.5,
        (left[3] + right[3]) * 0.5,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorBand;

    #[test]
    fn material_sprite_channels_resolve_by_band() {
        let color = CellColor::Material(CellMaterialId::GrayScale);
        assert_eq!(
            color.resolve_sprite(SpriteColorChannel::A, ColorBand::MediumDark),
            [
                0x55 as f32 / 255.0,
                0x55 as f32 / 255.0,
                0x55 as f32 / 255.0,
                1.0
            ]
        );
    }

    #[test]
    fn mixed_sprite_channels_blend_the_resolved_outputs() {
        let color = CellColor::slots(
            CellColorSlot::Flat([1.0, 0.0, 0.0, 1.0]),
            CellColorSlot::Flat([0.0, 1.0, 0.0, 1.0]),
            CellColorSlot::Flat([0.0, 0.0, 1.0, 1.0]),
        );

        assert_eq!(
            color.resolve_sprite(SpriteColorChannel::AB, ColorBand::Lightest),
            [0.5, 0.5, 0.0, 1.0]
        );
        assert_eq!(
            color.resolve_sprite(SpriteColorChannel::CA, ColorBand::Lightest),
            [0.5, 0.0, 0.5, 1.0]
        );
    }

    #[test]
    fn glyphs_use_slot_a_medium_light_resolution() {
        let color = CellColor::Material(CellMaterialId::GrayScale);
        assert_eq!(
            color.resolve_glyph(),
            [
                0xa8 as f32 / 255.0,
                0xa8 as f32 / 255.0,
                0xa8 as f32 / 255.0,
                1.0
            ]
        );
    }
}
