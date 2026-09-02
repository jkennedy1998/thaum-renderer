#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBand {
    Darkest,
    MediumDark,
    MediumLight,
    Lightest,
}

impl ColorBand {
    pub const fn as_index(self) -> usize {
        match self {
            Self::Darkest => 0,
            Self::MediumDark => 1,
            Self::MediumLight => 2,
            Self::Lightest => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellMaterialId {
    GrayScale,
}

impl CellMaterialId {
    pub const fn all() -> &'static [Self] {
        &[Self::GrayScale]
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::GrayScale => "gray-scale",
        }
    }

    pub fn resolve_band(self, band: ColorBand) -> [f32; 4] {
        match self {
            Self::GrayScale => GRAY_SCALE_BANDS[band.as_index()],
        }
    }

    pub fn resolve_glyph_default(self) -> [f32; 4] {
        self.resolve_band(ColorBand::MediumLight)
    }
}

const GRAY_SCALE_BANDS: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 1.0],
    [
        0x55 as f32 / 255.0,
        0x55 as f32 / 255.0,
        0x55 as f32 / 255.0,
        1.0,
    ],
    [
        0xa8 as f32 / 255.0,
        0xa8 as f32 / 255.0,
        0xa8 as f32 / 255.0,
        1.0,
    ],
    [1.0, 1.0, 1.0, 1.0],
];
