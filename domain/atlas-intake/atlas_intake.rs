use std::{
    fs::File,
    path::{Path, PathBuf},
};

use png::Decoder;

use crate::CellWeight;

pub const ATLAS_TILE_WIDTH: u32 = 12;
pub const ATLAS_TILE_HEIGHT: u32 = 16;

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

const fn weight_row_top_to_bottom(weight: CellWeight) -> u32 {
    match weight {
        CellWeight::Three => 0,
        CellWeight::Two => 1,
        CellWeight::One => 2,
        CellWeight::Zero => 3,
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
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::Three), 0);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::Two), 1);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::One), 2);
        assert_eq!(AtlasForm::Single.weight_row(CellWeight::Zero), 3);
        assert_eq!(AtlasForm::ConnectingCardinal.weight_row(CellWeight::Two), 4);
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
}
