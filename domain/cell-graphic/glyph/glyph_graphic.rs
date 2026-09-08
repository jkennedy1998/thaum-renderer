use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
};

use fontdue::{Font, FontSettings};
use png::{BitDepth, ColorType, Decoder};

#[cfg(test)]
use png::Encoder;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::CellWeight;

pub const GLYPH_TILE_WIDTH: usize = 12;
pub const GLYPH_TILE_HEIGHT: usize = 16;
const GLYPH_SPRITE_BATCH_ROOT: &str = "cell-sprites/monothaum-atlas-v3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphTileRaster {
    pub width: usize,
    pub height: usize,
    pub alpha: [u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
}

#[derive(Debug)]
pub struct GlyphFontSet {
    fonts: Option<[Font; 4]>,
    sprite_tiles: HashMap<(char, usize), GlyphTileRaster>,
}

impl GlyphTileRaster {
    fn empty() -> Self {
        Self {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [0; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
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
    pub fn load_from_asset_root(asset_root: &Path) -> Result<Self, String> {
        let sprite_tiles = load_glyph_sprite_tiles_from_asset_root(asset_root)?;
        let fonts = try_load_font_set(asset_root)?;

        if fonts.is_none() && sprite_tiles.is_empty() {
            return Err(format!(
                "failed to load glyph sources from {}: no font set and no sprite-batch glyph tiles found",
                asset_root.display()
            ));
        }

        Ok(Self {
            fonts,
            sprite_tiles,
        })
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
        if let Some(tile) = self.sprite_tiles.get(&(glyph, weight.as_index())) {
            return tile.clone();
        }

        let Some(fonts) = &self.fonts else {
            return GlyphTileRaster::empty();
        };

        let font = &fonts[weight.as_index()];
        let size = select_glyph_font_size(font, glyph);
        let (metrics, bitmap) = font.rasterize(glyph, size);
        let mut tile = GlyphTileRaster::empty();

        if metrics.width == 0 || metrics.height == 0 {
            return tile;
        }

        let line_metrics = font.horizontal_line_metrics(size);
        let advance_width = metrics.advance_width.ceil().max(metrics.width as f32);
        let baseline_y = line_metrics
            .map(|line| {
                ((GLYPH_TILE_HEIGHT as f32 - line.new_line_size) * 0.5 + line.ascent).round() as i32
            })
            .unwrap_or(GLYPH_TILE_HEIGHT as i32);
        let origin_x = ((GLYPH_TILE_WIDTH as f32 - advance_width) * 0.5)
            .floor()
            .max(0.0) as i32;
        let bitmap_left = origin_x + metrics.xmin;
        let bitmap_top = baseline_y - metrics.ymin - metrics.height as i32;

        for source_y in 0..metrics.height {
            for source_x in 0..metrics.width {
                let tile_x = bitmap_left + source_x as i32;
                let tile_y = bitmap_top + source_y as i32;
                if tile_x < 0
                    || tile_y < 0
                    || tile_x >= GLYPH_TILE_WIDTH as i32
                    || tile_y >= GLYPH_TILE_HEIGHT as i32
                {
                    continue;
                }

                let source_index = source_y * metrics.width + source_x;
                let tile_index = tile_y as usize * GLYPH_TILE_WIDTH + tile_x as usize;
                tile.alpha[tile_index] = bitmap[source_index];
            }
        }

        tile
    }
}

pub fn glyph_font_path(asset_root: &Path, weight: CellWeight) -> PathBuf {
    let file_name = match weight {
        CellWeight::Zero => "ThaumMono-W80.ttf",
        CellWeight::One => "ThaumMono-W160.ttf",
        CellWeight::Two => "ThaumMono-W320.ttf",
        CellWeight::Three => "ThaumMono-W640.ttf",
    };

    asset_root
        .join("glyph-fonts")
        .join("thaum-mono")
        .join(file_name)
}

fn try_load_font_set(asset_root: &Path) -> Result<Option<[Font; 4]>, String> {
    let paths = [
        glyph_font_path(asset_root, CellWeight::Zero),
        glyph_font_path(asset_root, CellWeight::One),
        glyph_font_path(asset_root, CellWeight::Two),
        glyph_font_path(asset_root, CellWeight::Three),
    ];

    let existing_count = paths.iter().filter(|path| path.exists()).count();
    if existing_count == 0 {
        return Ok(None);
    }

    if existing_count != paths.len() {
        return Err(format!(
            "glyph font asset root {} has a partial thaum mono font set",
            asset_root.display()
        ));
    }

    Ok(Some([
        load_font(&paths[0])?,
        load_font(&paths[1])?,
        load_font(&paths[2])?,
        load_font(&paths[3])?,
    ]))
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

fn select_glyph_font_size(font: &Font, glyph: char) -> f32 {
    for size in (1..=64).rev() {
        let size = size as f32;
        let metrics = font.metrics(glyph, size);
        let line_fits = font
            .horizontal_line_metrics(size)
            .map(|line| line.new_line_size.ceil() <= GLYPH_TILE_HEIGHT as f32)
            .unwrap_or(metrics.height <= GLYPH_TILE_HEIGHT);
        let advance_fits = metrics.advance_width.ceil() <= GLYPH_TILE_WIDTH as f32;

        if line_fits && advance_fits {
            return size;
        }

        if metrics.width <= GLYPH_TILE_WIDTH && metrics.height <= GLYPH_TILE_HEIGHT {
            return size;
        }
    }

    16.0
}

fn load_glyph_sprite_tiles_from_asset_root(
    asset_root: &Path,
) -> Result<HashMap<(char, usize), GlyphTileRaster>, String> {
    let batch_root = asset_root.join(GLYPH_SPRITE_BATCH_ROOT);
    if !batch_root.exists() {
        return Ok(HashMap::new());
    }

    let batch_ids = available_sprite_batch_ids(&batch_root)?;
    if batch_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let sections = parse_glyph_sprite_sections(&batch_root.join("sections.txt"), &batch_ids)?;
    let mut tiles = HashMap::new();

    for batch_id in batch_ids {
        let glyphs = sections.get(&batch_id).cloned().unwrap_or_default();
        let sheet = load_rgba_sheet(&batch_root.join(format!("monothaum_{batch_id}.png")))?;
        let sheet_columns = sheet.width / GLYPH_TILE_WIDTH;

        if glyphs.len() > sheet_columns {
            return Err(format!(
                "glyph sprite batch '{}' declares {} glyphs but only has {} columns",
                batch_id,
                glyphs.len(),
                sheet_columns
            ));
        }

        for (column_index, glyph) in glyphs.into_iter().enumerate() {
            for row_index in 0..4 {
                let weight_index = 3 - row_index;
                tiles.insert(
                    (glyph, weight_index),
                    extract_glyph_tile_from_sheet(&sheet, column_index, row_index),
                );
            }
        }
    }

    Ok(tiles)
}

fn available_sprite_batch_ids(batch_root: &Path) -> Result<Vec<String>, String> {
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
        if let Some(batch_id) = file_stem.strip_prefix("monothaum_") {
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
struct RgbaSheet {
    width: usize,
    rgba: Vec<u8>,
}

fn load_rgba_sheet(path: &Path) -> Result<RgbaSheet, String> {
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

    if info.color_type != ColorType::Rgba || info.bit_depth != BitDepth::Eight {
        return Err(format!(
            "unsupported glyph sprite sheet format at {}: expected 8-bit RGBA, found {:?} {:?}",
            path.display(),
            info.color_type,
            info.bit_depth
        ));
    }

    let width = info.width as usize;
    let height = info.height as usize;
    if width % GLYPH_TILE_WIDTH != 0 || height != GLYPH_TILE_HEIGHT * 4 {
        return Err(format!(
            "glyph sprite sheet at {} must be Nx64 with a 12px column grid; found {}x{}",
            path.display(),
            width,
            height
        ));
    }

    buffer.truncate(info.buffer_size());
    Ok(RgbaSheet {
        width,
        rgba: buffer,
    })
}

fn extract_glyph_tile_from_sheet(
    sheet: &RgbaSheet,
    column_index: usize,
    row_index: usize,
) -> GlyphTileRaster {
    let mut tile = GlyphTileRaster::empty();
    let origin_x = column_index * GLYPH_TILE_WIDTH;
    let origin_y = row_index * GLYPH_TILE_HEIGHT;

    for y in 0..GLYPH_TILE_HEIGHT {
        for x in 0..GLYPH_TILE_WIDTH {
            let pixel_index = ((origin_y + y) * sheet.width + (origin_x + x)) * 4;
            let alpha = sheet.rgba[pixel_index + 3];
            tile.alpha[y * GLYPH_TILE_WIDTH + x] = alpha;
        }
    }

    tile
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
    fn glyph_font_paths_follow_thaum_mono_weight_file_layout() {
        let asset_root = staged_asset_root();

        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Zero),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W80.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::One),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W160.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Two),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W320.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Three),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W640.ttf")
        );
    }

    #[test]
    fn rasterized_glyph_tiles_use_the_shared_tile_shape() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let raster = fonts.rasterize_glyph_tile('A', CellWeight::Two);

        assert_eq!(raster.width, GLYPH_TILE_WIDTH);
        assert_eq!(raster.height, GLYPH_TILE_HEIGHT);
        assert!(raster.coverage_count() > 0);
    }

    #[test]
    fn staged_asset_root_includes_sprite_batch_glyph_tiles() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        assert!(!fonts.sprite_tiles.is_empty());
    }

    #[test]
    fn sprite_batch_glyph_tiles_can_load_without_font_files() {
        let asset_root = test_temp_dir("glyph-sprite-batch");
        let batch_root = asset_root.join(GLYPH_SPRITE_BATCH_ROOT);
        fs::create_dir_all(&batch_root).unwrap();
        fs::write(batch_root.join("sections.txt"), "ascii:\nAB\n").unwrap();

        let mut rgba = vec![0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT * 2 * 4 * 4];
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[0] = 255;
            pixel[1] = 255;
            pixel[2] = 255;
            pixel[3] = 0;
        }
        let row_index = 1usize;
        let a_pixel_index = ((row_index * GLYPH_TILE_HEIGHT + 2) * (GLYPH_TILE_WIDTH * 2) + 3) * 4;
        let b_pixel_index =
            ((row_index * GLYPH_TILE_HEIGHT + 4) * (GLYPH_TILE_WIDTH * 2) + GLYPH_TILE_WIDTH + 5)
                * 4;
        rgba[a_pixel_index + 3] = 255;
        rgba[b_pixel_index + 3] = 255;
        write_test_rgba_png(
            &batch_root.join("monothaum_ascii.png"),
            (GLYPH_TILE_WIDTH * 2) as u32,
            (GLYPH_TILE_HEIGHT * 4) as u32,
            &rgba,
        );

        let fonts = GlyphFontSet::load_from_asset_root(&asset_root).unwrap();
        let a_raster = fonts.rasterize_glyph_tile('A', CellWeight::Two);
        let b_raster = fonts.rasterize_glyph_tile('B', CellWeight::Two);

        assert_eq!(a_raster.width, GLYPH_TILE_WIDTH);
        assert_eq!(a_raster.height, GLYPH_TILE_HEIGHT);
        assert_eq!(a_raster.coverage_count(), 1);
        assert_eq!(a_raster.alpha[2 * GLYPH_TILE_WIDTH + 3], 255);
        assert_eq!(a_raster.alpha[0], 0);
        assert_eq!(b_raster.coverage_count(), 1);
        assert_eq!(b_raster.alpha[4 * GLYPH_TILE_WIDTH + 5], 255);
        assert_eq!(b_raster.alpha[2 * GLYPH_TILE_WIDTH + 3], 0);
    }

    #[test]
    fn glyph_sprite_sections_preserve_the_leading_space_glyph() {
        let path = test_temp_dir("glyph-sections").join("sections.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "ascii:\n ABC\n").unwrap();

        let sections = parse_glyph_sprite_sections(&path, &["ascii".to_string()]).unwrap();
        let glyphs = sections.get("ascii").unwrap();

        assert_eq!(glyphs[0], ' ');
        assert_eq!(glyphs[1], 'A');
        assert_eq!(glyphs[2], 'B');
        assert_eq!(glyphs[3], 'C');
    }
}
