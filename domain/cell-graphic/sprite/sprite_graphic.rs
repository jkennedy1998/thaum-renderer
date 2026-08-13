use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    decode_sprite_rgba, load_sprite_atlas_image, CellColor, CellWeight, SpriteAtlasImage,
    GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteTileRaster {
    pub width: usize,
    pub height: usize,
    pub colors: Vec<Option<[f32; 4]>>,
}

#[derive(Debug)]
pub struct SpriteAtlasSet {
    atlas_root: PathBuf,
    atlases: HashMap<PathBuf, SpriteAtlasImage>,
}

impl SpriteAtlasSet {
    pub fn load_from_asset_root(asset_root: &Path) -> Self {
        Self {
            atlas_root: asset_root.join("cell-sprites"),
            atlases: HashMap::new(),
        }
    }

    pub fn rasterize_single_sprite_tile(
        &mut self,
        atlas_relative_path: &Path,
        weight: CellWeight,
        color: CellColor,
    ) -> Result<SpriteTileRaster, String> {
        let atlas = self.load_atlas(atlas_relative_path)?;
        let rgba = atlas.single_tile_rgba(weight)?;
        let decoded = decode_sprite_rgba(&rgba)?;
        let mut colors = Vec::with_capacity(GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT);

        for pixel in decoded {
            colors.push(pixel.map(|pixel| {
                let mut resolved = color.resolve_sprite(pixel.channel, pixel.band);
                resolved[3] *= pixel.alpha as f32 / 255.0;
                resolved
            }));
        }

        Ok(SpriteTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            colors,
        })
    }

    fn load_atlas(&mut self, atlas_relative_path: &Path) -> Result<&SpriteAtlasImage, String> {
        if !self.atlases.contains_key(atlas_relative_path) {
            let atlas_path = self.atlas_root.join(atlas_relative_path);
            let atlas = load_sprite_atlas_image(&atlas_path)?;
            self.atlases
                .insert(atlas_relative_path.to_path_buf(), atlas);
        }

        self.atlases.get(atlas_relative_path).ok_or_else(|| {
            format!(
                "sprite atlas cache lookup should succeed after load for {}",
                atlas_relative_path.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CellMaterialId;

    fn staged_asset_root() -> PathBuf {
        if let Ok(path) = std::env::var("THAUM_RENDERER_ASSET_ROOT") {
            return PathBuf::from(path);
        }

        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestration/renderer-assets")
    }

    #[test]
    fn sprite_atlas_set_uses_the_shared_tile_shape() {
        let mut atlases = SpriteAtlasSet::load_from_asset_root(&staged_asset_root());
        let raster = atlases
            .rasterize_single_sprite_tile(
                Path::new("proofs/grass.png"),
                CellWeight::Two,
                CellColor::Material(CellMaterialId::GrayScale),
            )
            .unwrap();

        assert_eq!(raster.width, GLYPH_TILE_WIDTH);
        assert_eq!(raster.height, GLYPH_TILE_HEIGHT);
        assert_eq!(raster.colors.len(), GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT);
        assert!(raster.colors.iter().any(|pixel| pixel.is_some()));
    }
}
