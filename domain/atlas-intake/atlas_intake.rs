use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
};

use png::Decoder;

#[cfg(test)]
use png::{BitDepth, ColorType, Encoder};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::CellWeight;

pub const ATLAS_TILE_WIDTH: u32 = 12;
pub const ATLAS_TILE_HEIGHT: u32 = 16;
const GLYPH_SPRITE_BATCH_SHEET_PREFIX: &str = "monothaum_";
const GLYPH_SPRITE_BATCH_WEIGHT_ROWS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasForm {
    Single,
    SixSided,
    ConnectingCardinal,
}

impl AtlasForm {
    pub const fn tile_columns(self) -> u32 {
        match self {
            Self::Single => 1,
            Self::SixSided => 6,
            Self::ConnectingCardinal => 4,
        }
    }

    pub const fn tile_rows(self) -> u32 {
        match self {
            Self::Single => 4,
            Self::SixSided => 4,
            Self::ConnectingCardinal => 16,
        }
    }

    pub const fn pixel_size(self) -> [u32; 2] {
        [
            self.tile_columns() * ATLAS_TILE_WIDTH,
            self.tile_rows() * ATLAS_TILE_HEIGHT,
        ]
    }

    pub const fn weight_row(self, weight: CellWeight) -> u32 {
        match self {
            Self::Single | Self::SixSided => weight_row_top_to_bottom(weight),
            Self::ConnectingCardinal => weight_row_top_to_bottom(weight) * 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtlasSpec {
    pub path: PathBuf,
    pub form: AtlasForm,
    pub image_width: u32,
    pub image_height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteAtlasImage {
    pub spec: AtlasSpec,
    pub rgba: Vec<u8>,
}

/// One alpha-only tile decoded from a named glyph spriteset. This is atlas
/// data, not a glyph raster: `cell-graphic/glyph/` decides how to merge it
/// with a selected typeface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphSpriteBatchTile {
    pub width: usize,
    pub height: usize,
    pub alpha: Vec<u8>,
}

/// A decoded named glyph spriteset. The set reports its own native cell
/// footprint so callers can later decide whether it matches or should stretch
/// to the active file's cell profile; atlas intake never rejects a mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphSpriteBatch {
    pub id: String,
    pub native_tile_width: usize,
    pub native_tile_height: usize,
    pub native_weight_count: usize,
    tiles: HashMap<(char, usize), GlyphSpriteBatchTile>,
}

impl GlyphSpriteBatch {
    pub fn tile(&self, glyph: char, weight: CellWeight) -> Option<&GlyphSpriteBatchTile> {
        self.tiles.get(&(glyph, weight.as_index()))
    }

    pub fn into_tiles(self) -> HashMap<(char, usize), GlyphSpriteBatchTile> {
        self.tiles
    }
}

impl AtlasSpec {
    pub const fn tile_count(&self) -> [u32; 2] {
        [
            self.image_width / self.tile_width,
            self.image_height / self.tile_height,
        ]
    }
}

impl SpriteAtlasImage {
    pub fn single_tile_rgba(&self, weight: CellWeight) -> Result<Vec<u8>, String> {
        if self.spec.form != AtlasForm::Single {
            return Err(format!(
                "single_tile_rgba expected a single atlas, found {:?}",
                self.spec.form
            ));
        }

        self.tile_rgba(0, self.spec.form.weight_row(weight))
    }

    pub fn tile_rgba(&self, tile_column: u32, tile_row: u32) -> Result<Vec<u8>, String> {
        let tile_counts = self.spec.tile_count();
        if tile_column >= tile_counts[0] || tile_row >= tile_counts[1] {
            return Err(format!(
                "tile lookup {tile_column},{tile_row} is outside atlas tile bounds {}x{}",
                tile_counts[0], tile_counts[1]
            ));
        }

        let mut tile = vec![0; (self.spec.tile_width * self.spec.tile_height * 4) as usize];
        let image_width = self.spec.image_width as usize;
        let tile_width = self.spec.tile_width as usize;
        let tile_height = self.spec.tile_height as usize;
        let origin_x = tile_column as usize * tile_width;
        let origin_y = tile_row as usize * tile_height;

        for row in 0..tile_height {
            for column in 0..tile_width {
                let source_pixel = ((origin_y + row) * image_width + (origin_x + column)) * 4;
                let target_pixel = (row * tile_width + column) * 4;
                tile[target_pixel..target_pixel + 4]
                    .copy_from_slice(&self.rgba[source_pixel..source_pixel + 4]);
            }
        }

        Ok(tile)
    }
}

pub fn load_sprite_atlas_spec(path: impl Into<PathBuf>) -> Result<AtlasSpec, String> {
    let path = path.into();
    let (image_width, image_height) = read_png_dimensions(&path)?;
    let form = infer_atlas_form_from_dimensions(image_width, image_height)?;

    Ok(AtlasSpec {
        path,
        form,
        image_width,
        image_height,
        tile_width: ATLAS_TILE_WIDTH,
        tile_height: ATLAS_TILE_HEIGHT,
    })
}

pub fn load_sprite_atlas_image(path: impl Into<PathBuf>) -> Result<SpriteAtlasImage, String> {
    let path = path.into();
    let spec = load_sprite_atlas_spec(path.clone())?;
    let rgba = read_png_rgba(&path)?;

    if rgba.len() != (spec.image_width * spec.image_height * 4) as usize {
        return Err(format!(
            "atlas rgba size mismatch for {}: expected {} bytes, got {}",
            path.display(),
            spec.image_width * spec.image_height * 4,
            rgba.len()
        ));
    }

    Ok(SpriteAtlasImage { spec, rgba })
}

/// Decode one explicitly selected glyph spriteset below an asset root.
///
/// The current on-disk `sections.txt` + `monothaum_<section>.png` convention
/// is preserved exactly while its ownership moves out of glyph rasterization.
/// A missing set is not an error yet: legacy callers may have font-only
/// assets. A selected file profile can turn that absence into its own refusal
/// once phase 8 owns the profile selection.
pub fn load_glyph_sprite_batch_from_asset_root(
    asset_root: &Path,
    spriteset_id: &str,
) -> Result<Option<GlyphSpriteBatch>, String> {
    let id = spriteset_id.trim();
    if id.is_empty() {
        return Err("glyph spriteset id must not be empty".to_owned());
    }

    let batch_root = asset_root.join("cell-sprites").join(id);
    if !batch_root.exists() {
        return Ok(None);
    }

    let batch_ids = available_glyph_sprite_batch_ids(&batch_root)?;
    if batch_ids.is_empty() {
        return Ok(Some(GlyphSpriteBatch {
            id: id.to_owned(),
            native_tile_width: ATLAS_TILE_WIDTH as usize,
            native_tile_height: ATLAS_TILE_HEIGHT as usize,
            native_weight_count: GLYPH_SPRITE_BATCH_WEIGHT_ROWS,
            tiles: HashMap::new(),
        }));
    }

    let sections = parse_glyph_sprite_sections(&batch_root.join("sections.txt"), &batch_ids)?;
    let mut tiles = HashMap::new();

    for batch_id in batch_ids {
        let glyphs = sections.get(&batch_id).cloned().unwrap_or_default();
        let sheet = load_glyph_sprite_sheet(
            &batch_root.join(format!("{GLYPH_SPRITE_BATCH_SHEET_PREFIX}{batch_id}.png")),
        )?;
        let sheet_columns = sheet.width / ATLAS_TILE_WIDTH as usize;

        if glyphs.len() > sheet_columns {
            return Err(format!(
                "glyph sprite batch '{}' declares {} glyphs but only has {} columns",
                batch_id,
                glyphs.len(),
                sheet_columns
            ));
        }

        for (column_index, glyph) in glyphs.into_iter().enumerate() {
            for row_index in 0..GLYPH_SPRITE_BATCH_WEIGHT_ROWS {
                let weight_index = GLYPH_SPRITE_BATCH_WEIGHT_ROWS - 1 - row_index;
                tiles.insert(
                    (glyph, weight_index),
                    extract_glyph_sprite_tile(&sheet, column_index, row_index),
                );
            }
        }
    }

    Ok(Some(GlyphSpriteBatch {
        id: id.to_owned(),
        native_tile_width: ATLAS_TILE_WIDTH as usize,
        native_tile_height: ATLAS_TILE_HEIGHT as usize,
        native_weight_count: GLYPH_SPRITE_BATCH_WEIGHT_ROWS,
        tiles,
    }))
}

fn infer_atlas_form_from_dimensions(
    image_width: u32,
    image_height: u32,
) -> Result<AtlasForm, String> {
    const SINGLE_WIDTH: u32 = ATLAS_TILE_WIDTH;
    const SINGLE_HEIGHT: u32 = ATLAS_TILE_HEIGHT * 4;
    const SIX_SIDED_WIDTH: u32 = ATLAS_TILE_WIDTH * 6;
    const SIX_SIDED_HEIGHT: u32 = ATLAS_TILE_HEIGHT * 4;
    const CONNECTING_WIDTH: u32 = ATLAS_TILE_WIDTH * 4;
    const CONNECTING_HEIGHT: u32 = ATLAS_TILE_HEIGHT * 16;

    match (image_width, image_height) {
        (SINGLE_WIDTH, SINGLE_HEIGHT) => Ok(AtlasForm::Single),
        (SIX_SIDED_WIDTH, SIX_SIDED_HEIGHT) => Ok(AtlasForm::SixSided),
        (CONNECTING_WIDTH, CONNECTING_HEIGHT) => Ok(AtlasForm::ConnectingCardinal),
        _ => Err(format!(
            "unsupported atlas dimensions {image_width}x{image_height}; expected one of 12x64, 72x64, or 48x256"
        )),
    }
}

fn read_png_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let file = File::open(path)
        .map_err(|error| format!("failed to open atlas png at {}: {error}", path.display()))?;
    let decoder = Decoder::new(file);
    let reader = decoder.read_info().map_err(|error| {
        format!(
            "failed to read atlas png header at {}: {error}",
            path.display()
        )
    })?;
    let info = reader.info();
    Ok((info.width, info.height))
}

fn read_png_rgba(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path)
        .map_err(|error| format!("failed to open atlas png at {}: {error}", path.display()))?;
    let decoder = Decoder::new(file);
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("failed to read atlas png at {}: {error}", path.display()))?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("failed to decode atlas png at {}: {error}", path.display()))?;

    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "unsupported atlas png format at {}: expected 8-bit RGBA, found {:?} {:?}",
            path.display(),
            info.color_type,
            info.bit_depth
        ));
    }

    buffer.truncate(info.buffer_size());
    Ok(buffer)
}

fn available_glyph_sprite_batch_ids(batch_root: &Path) -> Result<Vec<String>, String> {
    let mut batch_ids = Vec::new();

    for entry in fs::read_dir(batch_root).map_err(|error| {
        format!(
            "failed to read glyph sprite batch root {}: {error}",
            batch_root.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read entry under glyph sprite batch root {}: {error}",
                batch_root.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("png") {
            continue;
        }

        let Some(file_stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if let Some(batch_id) = file_stem.strip_prefix(GLYPH_SPRITE_BATCH_SHEET_PREFIX) {
            batch_ids.push(batch_id.to_string());
        }
    }

    batch_ids.sort();
    Ok(batch_ids)
}

fn parse_glyph_sprite_sections(
    sections_path: &Path,
    available_batch_ids: &[String],
) -> Result<HashMap<String, Vec<char>>, String> {
    if !sections_path.exists() {
        return Ok(HashMap::new());
    }

    let known_batch_ids = available_batch_ids.iter().cloned().collect::<HashSet<_>>();
    let text = fs::read_to_string(sections_path).map_err(|error| {
        format!(
            "failed to read glyph sprite sections at {}: {error}",
            sections_path.display()
        )
    })?;

    let mut sections = HashMap::<String, String>::new();
    let mut current_batch_id = None::<String>;

    for line in text.lines() {
        if let Some((head, tail)) = line.split_once(':') {
            let batch_id = head.trim();
            if known_batch_ids.contains(batch_id) {
                current_batch_id = Some(batch_id.to_string());
                sections
                    .entry(batch_id.to_string())
                    .or_default()
                    .push_str(tail);
                continue;
            }
        }

        if line.is_empty() {
            continue;
        }

        if let Some(batch_id) = &current_batch_id {
            sections.entry(batch_id.clone()).or_default().push_str(line);
        }
    }

    Ok(sections
        .into_iter()
        .map(|(batch_id, glyphs)| (batch_id, glyphs.chars().collect()))
        .collect())
}

#[derive(Debug)]
struct GlyphSpriteSheet {
    width: usize,
    rgba: Vec<u8>,
}

fn load_glyph_sprite_sheet(path: &Path) -> Result<GlyphSpriteSheet, String> {
    let file = File::open(path).map_err(|error| {
        format!(
            "failed to open glyph sprite sheet at {}: {error}",
            path.display()
        )
    })?;
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info().map_err(|error| {
        format!(
            "failed to read glyph sprite sheet at {}: {error}",
            path.display()
        )
    })?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).map_err(|error| {
        format!(
            "failed to decode glyph sprite sheet at {}: {error}",
            path.display()
        )
    })?;

    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "unsupported glyph sprite sheet format at {}: expected 8-bit RGBA, found {:?} {:?}",
            path.display(),
            info.color_type,
            info.bit_depth
        ));
    }

    let width = info.width as usize;
    let height = info.height as usize;
    if width % ATLAS_TILE_WIDTH as usize != 0
        || height != ATLAS_TILE_HEIGHT as usize * GLYPH_SPRITE_BATCH_WEIGHT_ROWS
    {
        return Err(format!(
            "glyph sprite sheet at {} must be Nx64 with a 12px column grid; found {}x{}",
            path.display(),
            width,
            height
        ));
    }

    buffer.truncate(info.buffer_size());
    Ok(GlyphSpriteSheet {
        width,
        rgba: buffer,
    })
}

fn extract_glyph_sprite_tile(
    sheet: &GlyphSpriteSheet,
    column_index: usize,
    row_index: usize,
) -> GlyphSpriteBatchTile {
    let width = ATLAS_TILE_WIDTH as usize;
    let height = ATLAS_TILE_HEIGHT as usize;
    let origin_x = column_index * width;
    let origin_y = row_index * height;
    let mut alpha = vec![0; width * height];

    for y in 0..height {
        for x in 0..width {
            let pixel_index = ((origin_y + y) * sheet.width + (origin_x + x)) * 4;
            alpha[y * width + x] = sheet.rgba[pixel_index + 3];
        }
    }

    GlyphSpriteBatchTile {
        width,
        height,
        alpha,
    }
}

/// These stacked atlas forms (`single`, `six-sided`, `connecting-cardinal`)
/// are fixed 4-row shapes by their own declared form, independent of
/// `CellWeight`'s now-unbounded range. An index beyond row 3 clamps to row 0
/// (the highest/top row) rather than panicking.
const STACKED_ATLAS_FORM_MAX_WEIGHT_INDEX: usize = 3;

const fn weight_row_top_to_bottom(weight: CellWeight) -> u32 {
    let index = weight.as_index();
    if index >= STACKED_ATLAS_FORM_MAX_WEIGHT_INDEX {
        0
    } else {
        (STACKED_ATLAS_FORM_MAX_WEIGHT_INDEX - index) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staged_asset_root() -> PathBuf {
        if let Ok(path) = std::env::var("THAUM_RENDERER_ASSET_ROOT") {
            return PathBuf::from(path);
        }

        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestration/renderer-assets")
    }

    fn test_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("thaum-renderer-{label}-{unique}"))
    }

    fn write_test_rgba_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = File::create(path).unwrap();
        let mut encoder = Encoder::new(file, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(rgba).unwrap();
    }

    #[test]
    fn single_atlas_dimensions_map_to_single_form() {
        assert_eq!(
            infer_atlas_form_from_dimensions(ATLAS_TILE_WIDTH, ATLAS_TILE_HEIGHT * 4).unwrap(),
            AtlasForm::Single
        );
    }

    #[test]
    fn six_sided_atlas_dimensions_map_to_six_sided_form() {
        assert_eq!(
            infer_atlas_form_from_dimensions(ATLAS_TILE_WIDTH * 6, ATLAS_TILE_HEIGHT * 4).unwrap(),
            AtlasForm::SixSided
        );
    }

    #[test]
    fn connecting_cardinal_dimensions_map_to_connecting_cardinal_form() {
        assert_eq!(
            infer_atlas_form_from_dimensions(ATLAS_TILE_WIDTH * 4, ATLAS_TILE_HEIGHT * 16).unwrap(),
            AtlasForm::ConnectingCardinal
        );
    }

    #[test]
    fn weight_rows_follow_top_heaviest_to_bottom_lightest_order() {
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::THREE), 0);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::TWO), 1);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::ONE), 2);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::ZERO), 3);
        assert_eq!(AtlasForm::ConnectingCardinal.weight_row(CellWeight::TWO), 4);
    }

    #[test]
    fn staged_grass_proof_png_loads_as_single_atlas_spec() {
        let atlas =
            load_sprite_atlas_spec(staged_asset_root().join("cell-sprites/proofs/grass.png"))
                .unwrap();

        assert_eq!(atlas.form, AtlasForm::Single);
        assert_eq!(atlas.image_width, ATLAS_TILE_WIDTH);
        assert_eq!(atlas.image_height, ATLAS_TILE_HEIGHT * 4);
        assert_eq!(atlas.tile_count(), [1, 4]);
    }

    #[test]
    fn staged_channel_band_proof_png_loads_as_single_atlas_spec() {
        let atlas = load_sprite_atlas_spec(
            staged_asset_root().join("cell-sprites/proofs/channel-bands.png"),
        )
        .unwrap();

        assert_eq!(atlas.form, AtlasForm::Single);
        assert_eq!(atlas.image_width, ATLAS_TILE_WIDTH);
        assert_eq!(atlas.image_height, ATLAS_TILE_HEIGHT * 4);
        assert_eq!(atlas.tile_count(), [1, 4]);
    }

    #[test]
    fn staged_monothaum_spriteset_decodes_as_one_named_batch() {
        let batch =
            load_glyph_sprite_batch_from_asset_root(&staged_asset_root(), "monothaum-atlas-v3")
                .unwrap()
                .unwrap();

        assert_eq!(batch.id, "monothaum-atlas-v3");
        assert_eq!(batch.native_tile_width, ATLAS_TILE_WIDTH as usize);
        assert_eq!(batch.native_tile_height, ATLAS_TILE_HEIGHT as usize);
        assert_eq!(batch.native_weight_count, 4);
        assert!(batch.tile('A', CellWeight::TWO).is_some());
    }

    #[test]
    fn missing_named_spriteset_returns_none_for_a_font_only_asset_root() {
        let batch =
            load_glyph_sprite_batch_from_asset_root(&test_temp_dir("missing-spriteset"), "none")
                .unwrap();
        assert!(batch.is_none());
    }

    #[test]
    fn glyph_sprite_batch_preserves_sections_and_weight_rows() {
        let asset_root = test_temp_dir("glyph-sprite-batch");
        let batch_root = asset_root.join("cell-sprites/example");
        fs::create_dir_all(&batch_root).unwrap();
        fs::write(batch_root.join("sections.txt"), "ascii:\n AB\n").unwrap();

        let width = ATLAS_TILE_WIDTH as usize * 3;
        let height = ATLAS_TILE_HEIGHT as usize * GLYPH_SPRITE_BATCH_WEIGHT_ROWS;
        let mut rgba = vec![0u8; width * height * 4];
        let row_index = 1usize;
        let a_pixel_index =
            ((row_index * ATLAS_TILE_HEIGHT as usize + 2) * width + ATLAS_TILE_WIDTH as usize + 3)
                * 4;
        let b_pixel_index = ((row_index * ATLAS_TILE_HEIGHT as usize + 4) * width
            + ATLAS_TILE_WIDTH as usize * 2
            + 5)
            * 4;
        rgba[a_pixel_index + 3] = 255;
        rgba[b_pixel_index + 3] = 255;
        write_test_rgba_png(
            &batch_root.join("monothaum_ascii.png"),
            width as u32,
            height as u32,
            &rgba,
        );

        let batch = load_glyph_sprite_batch_from_asset_root(&asset_root, "example")
            .unwrap()
            .unwrap();
        let space_tile = batch.tile(' ', CellWeight::TWO).unwrap();
        let a_tile = batch.tile('A', CellWeight::TWO).unwrap();
        let b_tile = batch.tile('B', CellWeight::TWO).unwrap();

        assert_eq!(
            space_tile.alpha.iter().filter(|alpha| **alpha > 0).count(),
            0
        );
        assert_eq!(a_tile.alpha[2 * ATLAS_TILE_WIDTH as usize + 3], 255);
        assert_eq!(b_tile.alpha[4 * ATLAS_TILE_WIDTH as usize + 5], 255);
        assert_eq!(b_tile.alpha[2 * ATLAS_TILE_WIDTH as usize + 3], 0);
    }

    #[test]
    fn glyph_sprite_batch_rejects_a_partial_sheet() {
        let asset_root = test_temp_dir("partial-glyph-sprite-batch");
        let batch_root = asset_root.join("cell-sprites/example");
        fs::create_dir_all(&batch_root).unwrap();
        fs::write(batch_root.join("sections.txt"), "ascii:\nAB\n").unwrap();
        write_test_rgba_png(
            &batch_root.join("monothaum_ascii.png"),
            ATLAS_TILE_WIDTH,
            ATLAS_TILE_HEIGHT * GLYPH_SPRITE_BATCH_WEIGHT_ROWS as u32,
            &vec![
                0;
                ATLAS_TILE_WIDTH as usize
                    * ATLAS_TILE_HEIGHT as usize
                    * GLYPH_SPRITE_BATCH_WEIGHT_ROWS
                    * 4
            ],
        );

        let error = load_glyph_sprite_batch_from_asset_root(&asset_root, "example").unwrap_err();
        assert!(error.contains("declares 2 glyphs but only has 1 columns"));
    }
}
