#[path = "glyph/glyph_graphic.rs"]
pub mod glyph_graphic;
#[path = "sprite/sprite_graphic.rs"]
pub mod sprite_graphic;

use std::path::{Path, PathBuf};

pub use glyph_graphic::{
    glyph_font_path, GlyphFontSet, GlyphTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH,
};
pub use sprite_graphic::{SpriteAtlasSet, SpriteTileRaster};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellGraphic {
    None,
    Glyph(char),
    Sprite(SpriteGraphic),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteGraphic {
    atlas_relative_path: PathBuf,
}

impl SpriteGraphic {
    pub fn new(atlas_relative_path: impl Into<PathBuf>) -> Self {
        Self {
            atlas_relative_path: atlas_relative_path.into(),
        }
    }

    pub fn atlas_relative_path(&self) -> &Path {
        &self.atlas_relative_path
    }
}

impl Default for CellGraphic {
    fn default() -> Self {
        Self::None
    }
}

impl CellGraphic {
    pub const fn is_visible(&self) -> bool {
        match self {
            Self::None => false,
            Self::Glyph(glyph) => *glyph != ' ',
            Self::Sprite(_) => true,
        }
    }

    pub const fn glyph_char(&self) -> Option<char> {
        match self {
            Self::None => None,
            Self::Glyph(' ') => None,
            Self::Glyph(glyph) => Some(*glyph),
            Self::Sprite(_) => None,
        }
    }

    pub fn sprite(&self) -> Option<&SpriteGraphic> {
        match self {
            Self::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_and_sprites_share_one_visibility_boundary() {
        assert!(!CellGraphic::None.is_visible());
        assert!(!CellGraphic::Glyph(' ').is_visible());
        assert!(CellGraphic::Glyph('@').is_visible());
        assert!(CellGraphic::Sprite(SpriteGraphic::new("proofs/grass.png")).is_visible());
    }

    #[test]
    fn graphic_accessors_keep_glyph_and_sprite_modes_separate() {
        let glyph = CellGraphic::Glyph('A');
        let sprite = CellGraphic::Sprite(SpriteGraphic::new("proofs/grass.png"));

        assert_eq!(glyph.glyph_char(), Some('A'));
        assert!(glyph.sprite().is_none());
        assert!(sprite.glyph_char().is_none());
        assert_eq!(
            sprite.sprite().unwrap().atlas_relative_path(),
            Path::new("proofs/grass.png")
        );
    }
}
