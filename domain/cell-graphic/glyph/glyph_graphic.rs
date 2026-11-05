use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use fontdue::{Font, FontSettings};

use super::typeface_intake::{load_typeface_declaration, TypefaceDeclaration, TypefaceSpacing};
use crate::{
    atlas_intake::{load_glyph_sprite_batch_from_asset_root, GlyphSpriteBatchTile},
    CellWeight,
};

/// Compatibility footprint for callers not yet wired to the per-file display
/// profile. Glyph rasterization itself no longer uses these constants: new
/// callers supply a [`GlyphCellProfile`].
// TODO(project 04n3fq3e phase-7): remove after boot owns the file profile.
pub const GLYPH_TILE_WIDTH: usize = 12;
// TODO(project 04n3fq3e phase-7): remove after boot owns the file profile.
pub const GLYPH_TILE_HEIGHT: usize = 16;
const LEGACY_TYPEFACE_DECLARATION: &str = "typefaces/thaum-mono.json";
const LEGACY_SPRITESET_ID: &str = "monothaum-atlas-v3";

/// The externally selected render-cell footprint. It belongs to a file's
/// display profile, not to the typeface declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphCellProfile {
    width: usize,
    height: usize,
}

impl GlyphCellProfile {
    pub fn new(width: usize, height: usize) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err(
                "glyph cell profile width and height must both be greater than zero".to_owned(),
            );
        }
        if width.checked_mul(height).is_none() {
            return Err("glyph cell profile pixel count is too large".to_owned());
        }
        Ok(Self { width, height })
    }

    pub const fn legacy_thaum_mono() -> Self {
        Self {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
        }
    }

    pub const fn width(self) -> usize {
        self.width
    }

    pub const fn height(self) -> usize {
        self.height
    }

    fn pixel_count(self) -> usize {
        self.width * self.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphTileRaster {
    pub width: usize,
    pub height: usize,
    pub alpha: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GlyphFontSet {
    fonts: Vec<Font>,
    spacing: TypefaceSpacing,
    cell_profile: GlyphCellProfile,
    face_font_sizes: Vec<f32>,
    sprite_tiles: HashMap<(char, usize), GlyphSpriteBatchTile>,
    sprite_weight_count: usize,
}

impl GlyphTileRaster {
    pub fn empty(cell_profile: GlyphCellProfile) -> Self {
        Self {
            width: cell_profile.width,
            height: cell_profile.height,
            alpha: vec![0; cell_profile.pixel_count()],
        }
    }

    pub fn coverage_count(&self) -> usize {
        self.alpha.iter().filter(|value| **value > 0).count()
    }

    pub fn coverage_bounds(&self) -> Option<(usize, usize, usize, usize)> {
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut any = false;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.alpha[y * self.width + x] == 0 {
                    continue;
                }

                any = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        any.then_some((min_x, min_y, max_x, max_y))
    }
}

impl GlyphFontSet {
    /// Temporary compatibility loader for renderer boot before phase 7 wires
    /// the per-file profile through. Font paths still come from the parsed
    /// declaration rather than a glyph-local hardcoded filename ladder.
    pub fn load_from_asset_root(asset_root: &Path) -> Result<Self, String> {
        let declaration_path = asset_root.join(LEGACY_TYPEFACE_DECLARATION);
        let declaration = load_typeface_declaration(&declaration_path)
            .map_err(|error| format!("failed to load default typeface declaration: {error}"))?;
        Self::load_from_declaration(
            asset_root,
            &declaration,
            GlyphCellProfile::legacy_thaum_mono(),
            Some(LEGACY_SPRITESET_ID),
        )
    }

    /// Resolve the named declaration selected by a file/boot display profile.
    /// Relative font paths are rooted at `asset_root`; absolute declaration
    /// paths remain external user/system font paths.
    pub fn load_selected_from_asset_root(
        asset_root: &Path,
        typeface_id: &str,
        cell_profile: GlyphCellProfile,
        spriteset_id: Option<&str>,
    ) -> Result<Self, String> {
        if typeface_id.is_empty()
            || !typeface_id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err("typeface id must contain only lowercase letters, digits, and hyphens".to_owned());
        }
        let declaration_path = asset_root
            .join("typefaces")
            .join(format!("{typeface_id}.json"));
        let declaration = load_typeface_declaration(&declaration_path)
            .map_err(|error| format!("failed to load typeface '{typeface_id}': {error}"))?;
        Self::load_from_declaration(asset_root, &declaration, cell_profile, spriteset_id)
    }

    /// Resolve one parsed declaration into one active font set. Relative font
    /// paths are rooted at `asset_root`; absolute declaration paths remain
    /// external user/system font paths. The optional spriteset is selected by
    /// id only—its native footprint is stretched to the requested profile and
    /// never used to reject or substitute a typeface.
    pub fn load_from_declaration(
        asset_root: &Path,
        declaration: &TypefaceDeclaration,
        cell_profile: GlyphCellProfile,
        spriteset_id: Option<&str>,
    ) -> Result<Self, String> {
        if declaration.weight_paths.is_empty() {
            return Err("typeface declaration must include at least one font weight".to_owned());
        }

        let fonts = declaration
            .weight_paths
            .iter()
            .map(|weight_path| load_font(&resolve_weight_path(asset_root, weight_path)))
            .collect::<Result<Vec<_>, _>>()?;
        let face_font_sizes = fonts
            .iter()
            .map(|font| select_face_font_size(font, declaration.spacing, cell_profile))
            .collect();
        let (sprite_tiles, sprite_weight_count) = match spriteset_id {
            Some(spriteset_id) => {
                match load_glyph_sprite_batch_from_asset_root(asset_root, spriteset_id)? {
                    Some(batch) => {
                        let weight_count = batch.native_weight_count;
                        (batch.into_tiles(), weight_count)
                    }
                    None => (HashMap::new(), 0),
                }
            }
            None => (HashMap::new(), 0),
        };

        Ok(Self {
            fonts,
            spacing: declaration.spacing,
            cell_profile,
            face_font_sizes,
            sprite_tiles,
            sprite_weight_count,
        })
    }

    /// Refit already-loaded font faces to a new file cell profile. This is a
    /// pure in-memory operation: a cell-size nudge does not reread font files
    /// or reload the active typeface.
    pub fn with_cell_profile(&self, cell_profile: GlyphCellProfile) -> Self {
        Self {
            fonts: self.fonts.clone(),
            spacing: self.spacing,
            cell_profile,
            face_font_sizes: self
                .fonts
                .iter()
                .map(|font| select_face_font_size(font, self.spacing, cell_profile))
                .collect(),
            sprite_tiles: self.sprite_tiles.clone(),
            sprite_weight_count: self.sprite_weight_count,
        }
    }

    pub const fn cell_profile(&self) -> GlyphCellProfile {
        self.cell_profile
    }

    pub fn native_weight_count(&self) -> usize {
        self.fonts.len()
    }

    /// Every char this set can contribute a tile for: the ASCII printable
    /// range (the font-backed charset) plus any sprite-batch glyphs, sorted
    /// and deduplicated. Tile-provider sweeps (shape-fade graph builds)
    /// consume this instead of re-deriving a charset.
    pub fn glyph_chars(&self) -> Vec<char> {
        let mut chars: Vec<char> = (' '..='~').collect();
        chars.extend(self.sprite_tiles.keys().map(|(glyph, _)| *glyph));
        chars.sort();
        chars.dedup();
        chars
    }

    pub fn rasterize_glyph_tile(&self, glyph: char, weight: CellWeight) -> GlyphTileRaster {
        if self.sprite_weight_count > 0 {
            let sprite_weight = weight
                .clamped_to_max((self.sprite_weight_count - 1).min(u8::MAX as usize) as u8)
                .as_index();
            if let Some(tile) = self.sprite_tiles.get(&(glyph, sprite_weight)) {
                return stretch_sprite_tile_to_profile(tile, self.cell_profile);
            }
        }

        let Some(font_index) = self.font_index_for_weight(weight) else {
            return GlyphTileRaster::empty(self.cell_profile);
        };
        rasterize_font_glyph(
            &self.fonts[font_index],
            self.face_font_sizes[font_index],
            self.spacing,
            self.cell_profile,
            glyph,
        )
    }

    fn font_index_for_weight(&self, weight: CellWeight) -> Option<usize> {
        (!self.fonts.is_empty()).then(|| weight.as_index().min(self.fonts.len() - 1))
    }
}

fn resolve_weight_path(asset_root: &Path, declared_path: &Path) -> PathBuf {
    if declared_path.is_absolute() {
        declared_path.to_path_buf()
    } else {
        asset_root.join(declared_path)
    }
}

fn load_font(path: &Path) -> Result<Font, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read glyph font asset at {}: {error}",
            path.display()
        )
    })?;
    Font::from_bytes(bytes, FontSettings::default()).map_err(|error| {
        format!(
            "failed to parse glyph font asset at {}: {}",
            path.display(),
            error
        )
    })
}

/// Choose one size per loaded face, not one size per glyph. Monospace faces
/// fit both line height and advance to the requested cell; proportional faces
/// fit their vertical line box and may overflow horizontally, where raster
/// placement deliberately clips them to their cell.
fn select_face_font_size(
    font: &Font,
    spacing: TypefaceSpacing,
    cell_profile: GlyphCellProfile,
) -> f32 {
    let largest_candidate = (cell_profile.width.max(cell_profile.height) * 4).clamp(1, 1024);
    let glyphs = font.chars().keys().copied().collect::<Vec<_>>();

    for size in (1..=largest_candidate).rev() {
        let size = size as f32;
        let line_fits = font
            .horizontal_line_metrics(size)
            .map(|line| line.new_line_size.ceil() <= cell_profile.height as f32)
            .unwrap_or_else(|| {
                glyphs
                    .iter()
                    .all(|glyph| font.metrics(*glyph, size).height <= cell_profile.height)
            });
        let advance_fits = match spacing {
            TypefaceSpacing::Monospace => glyphs.iter().all(|glyph| {
                font.metrics(*glyph, size).advance_width.ceil() <= cell_profile.width as f32
            }),
            TypefaceSpacing::Proportional => true,
        };

        if line_fits && advance_fits {
            return size;
        }
    }

    1.0
}

fn rasterize_font_glyph(
    font: &Font,
    size: f32,
    _spacing: TypefaceSpacing,
    cell_profile: GlyphCellProfile,
    glyph: char,
) -> GlyphTileRaster {
    let (metrics, bitmap) = font.rasterize(glyph, size);
    let mut tile = GlyphTileRaster::empty(cell_profile);

    if metrics.width == 0 || metrics.height == 0 {
        return tile;
    }

    let baseline_y = font
        .horizontal_line_metrics(size)
        .map(|line| {
            ((cell_profile.height as f32 - line.new_line_size) * 0.5 + line.ascent).round() as i32
        })
        .unwrap_or(cell_profile.height as i32);
    // Do not clamp the horizontal origin. A proportional glyph wider than its
    // cell stays baseline-centered and clips at the cell boundary rather than
    // shrinking, spilling into adjacent cells, or changing its raster size.
    let origin_x = ((cell_profile.width as f32 - metrics.advance_width) * 0.5).floor() as i32;
    let bitmap_left = origin_x + metrics.xmin;
    let bitmap_top = baseline_y - metrics.ymin - metrics.height as i32;

    for source_y in 0..metrics.height {
        for source_x in 0..metrics.width {
            let tile_x = bitmap_left + source_x as i32;
            let tile_y = bitmap_top + source_y as i32;
            if tile_x < 0
                || tile_y < 0
                || tile_x >= cell_profile.width as i32
                || tile_y >= cell_profile.height as i32
            {
                continue;
            }

            let source_index = source_y * metrics.width + source_x;
            let tile_index = tile_y as usize * cell_profile.width + tile_x as usize;
            tile.alpha[tile_index] = bitmap[source_index];
        }
    }

    tile
}

fn stretch_sprite_tile_to_profile(
    source: &GlyphSpriteBatchTile,
    cell_profile: GlyphCellProfile,
) -> GlyphTileRaster {
    let mut raster = GlyphTileRaster::empty(cell_profile);
    if source.width == 0 || source.height == 0 {
        return raster;
    }

    for target_y in 0..cell_profile.height {
        let source_y = target_y * source.height / cell_profile.height;
        for target_x in 0..cell_profile.width {
            let source_x = target_x * source.width / cell_profile.width;
            raster.alpha[target_y * cell_profile.width + target_x] =
                source.alpha[source_y * source.width + source_x];
        }
    }

    raster
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_typeface_declaration;

    fn staged_asset_root() -> PathBuf {
        if let Ok(path) = std::env::var("THAUM_RENDERER_ASSET_ROOT") {
            return PathBuf::from(path);
        }

        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestration/renderer-assets")
    }

    fn thaum_mono_declaration() -> TypefaceDeclaration {
        load_typeface_declaration(&staged_asset_root().join(LEGACY_TYPEFACE_DECLARATION)).unwrap()
    }

    #[test]
    fn glyph_cell_profile_rejects_zero_dimensions() {
        assert!(GlyphCellProfile::new(0, 16).is_err());
        assert!(GlyphCellProfile::new(12, 0).is_err());
    }

    #[test]
    fn declared_thaum_mono_rasterizes_into_the_requested_cell_profile() {
        let profile = GlyphCellProfile::new(20, 28).unwrap();
        let fonts = GlyphFontSet::load_from_declaration(
            &staged_asset_root(),
            &thaum_mono_declaration(),
            profile,
            Some(LEGACY_SPRITESET_ID),
        )
        .unwrap();
        let raster = fonts.rasterize_glyph_tile('A', CellWeight::TWO);

        assert_eq!(fonts.native_weight_count(), 4);
        assert_eq!(fonts.cell_profile(), profile);
        assert_eq!(raster.width, 20);
        assert_eq!(raster.height, 28);
        assert_eq!(raster.alpha.len(), 20 * 28);
        assert!(raster.coverage_count() > 0);
    }

    #[test]
    fn cell_size_nudge_reuses_loaded_faces_without_reloading_files() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let nudged = fonts.with_cell_profile(GlyphCellProfile::new(17, 23).unwrap());
        let raster = nudged.rasterize_glyph_tile('A', CellWeight::TWO);

        assert_eq!(nudged.native_weight_count(), fonts.native_weight_count());
        assert_eq!(raster.width, 17);
        assert_eq!(raster.height, 23);
        assert!(raster.coverage_count() > 0);
    }

    #[test]
    fn typeface_swap_loads_its_own_declared_weight_ladder() {
        let full_weight = thaum_mono_declaration();
        let single_weight = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": ["glyph-fonts/thaum-mono/ThaumMono-W80.ttf"]
            }"#,
        )
        .unwrap();
        let profile = GlyphCellProfile::legacy_thaum_mono();
        let full_fonts =
            GlyphFontSet::load_from_declaration(&staged_asset_root(), &full_weight, profile, None)
                .unwrap();
        let single_fonts = GlyphFontSet::load_from_declaration(
            &staged_asset_root(),
            &single_weight,
            profile,
            None,
        )
        .unwrap();

        assert_eq!(full_fonts.native_weight_count(), 4);
        assert_eq!(single_fonts.native_weight_count(), 1);
        assert!(
            single_fonts
                .rasterize_glyph_tile('A', CellWeight::from_index_clamped(40))
                .coverage_count()
                > 0
        );
    }

    #[test]
    fn selected_sprite_batch_stretches_under_sprite_over_font_precedence() {
        let profile = GlyphCellProfile::new(24, 16).unwrap();
        let sprite_fonts = GlyphFontSet::load_from_declaration(
            &staged_asset_root(),
            &thaum_mono_declaration(),
            profile,
            Some(LEGACY_SPRITESET_ID),
        )
        .unwrap();
        let font_only = GlyphFontSet::load_from_declaration(
            &staged_asset_root(),
            &thaum_mono_declaration(),
            profile,
            None,
        )
        .unwrap();
        let sprite = sprite_fonts.rasterize_glyph_tile('A', CellWeight::TWO);
        let font = font_only.rasterize_glyph_tile('A', CellWeight::TWO);

        assert_eq!(sprite.width, 24);
        assert_eq!(sprite.height, 16);
        assert!(sprite.coverage_count() > 0);
        assert_ne!(sprite.alpha, font.alpha);
    }

    #[test]
    fn proportional_overflow_is_clipped_to_the_target_cell() {
        let declaration = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "proportional",
                "weights": ["glyph-fonts/thaum-mono/ThaumMono-W640.ttf"]
            }"#,
        )
        .unwrap();
        let profile = GlyphCellProfile::new(1, 16).unwrap();
        let fonts =
            GlyphFontSet::load_from_declaration(&staged_asset_root(), &declaration, profile, None)
                .unwrap();
        let raster = fonts.rasterize_glyph_tile('W', CellWeight::ZERO);

        assert_eq!(raster.width, 1);
        assert_eq!(raster.height, 16);
        assert_eq!(raster.alpha.len(), 16);
        assert!(raster.coverage_count() > 0);
    }

    #[test]
    fn legacy_loader_keeps_the_default_thaum_mono_profile() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let raster = fonts.rasterize_glyph_tile('A', CellWeight::TWO);

        assert_eq!(raster.width, GLYPH_TILE_WIDTH);
        assert_eq!(raster.height, GLYPH_TILE_HEIGHT);
        assert!(raster.coverage_count() > 0);
    }
}
