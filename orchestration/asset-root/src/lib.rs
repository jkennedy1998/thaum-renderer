use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRootPaths {
    pub root: PathBuf,
    pub glyph_fonts: PathBuf,
    pub cell_sprites: PathBuf,
    pub materials: PathBuf,
    pub cell_shaders: PathBuf,
    pub cell_effects: PathBuf,
    pub post_effects: PathBuf,
}

pub fn resolve_asset_root_paths(asset_root: impl Into<PathBuf>) -> AssetRootPaths {
    let root = asset_root.into();

    AssetRootPaths {
        glyph_fonts: root.join("glyph-fonts"),
        cell_sprites: root.join("cell-sprites"),
        materials: root.join("materials"),
        cell_shaders: root.join("cell-shaders"),
        cell_effects: root.join("cell-effects"),
        post_effects: root.join("post-effects"),
        root,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_asset_root_paths_maps_expected_renderer_folders() {
        let resolved = resolve_asset_root_paths(PathBuf::from("renderer-assets"));

        assert_eq!(resolved.root, PathBuf::from("renderer-assets"));
        assert_eq!(
            resolved.glyph_fonts,
            PathBuf::from("renderer-assets/glyph-fonts")
        );
        assert_eq!(
            resolved.cell_sprites,
            PathBuf::from("renderer-assets/cell-sprites")
        );
        assert_eq!(
            resolved.materials,
            PathBuf::from("renderer-assets/materials")
        );
        assert_eq!(
            resolved.cell_shaders,
            PathBuf::from("renderer-assets/cell-shaders")
        );
        assert_eq!(
            resolved.cell_effects,
            PathBuf::from("renderer-assets/cell-effects")
        );
        assert_eq!(
            resolved.post_effects,
            PathBuf::from("renderer-assets/post-effects")
        );
    }
}
