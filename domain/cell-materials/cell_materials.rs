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

    /// Clamps an out-of-range index to the nearest end band rather than
    /// wrapping or panicking — this is the "losing detail" half of light
    /// shading: once a shift pushes past `Darkest`/`Lightest`, every further
    /// step collapses onto that same end band instead of adding new steps.
    pub const fn from_index_clamped(index: i32) -> Self {
        match index {
            i32::MIN..=0 => Self::Darkest,
            1 => Self::MediumDark,
            2 => Self::MediumLight,
            _ => Self::Lightest,
        }
    }
}

/// One pixel's position on the program-wide light ramp: brand black,
/// one of a material's four authored bands, or brand white. Light shifts
/// walk this complete six-value ramp so the outer colors can enter and leave
/// the material range instead of making black transparent or treating white
/// as a material value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexedColor {
    BrandBlack,
    Material(ColorBand),
    BrandWhite,
}

impl IndexedColor {
    const BRAND_BLACK_INDEX: i32 = -1;
    const BRAND_WHITE_INDEX: i32 = 4;

    const fn index(self) -> i32 {
        match self {
            Self::BrandBlack => Self::BRAND_BLACK_INDEX,
            Self::Material(band) => band.as_index() as i32,
            Self::BrandWhite => Self::BRAND_WHITE_INDEX,
        }
    }

    /// Moves through brand-black → material bands → brand-white, saturating
    /// at either program color. This is the color-merging behavior at the
    /// lighting extremes.
    pub fn shift(self, amount: i32) -> Self {
        match (self.index() + amount).clamp(Self::BRAND_BLACK_INDEX, Self::BRAND_WHITE_INDEX) {
            Self::BRAND_BLACK_INDEX => Self::BrandBlack,
            0 => Self::Material(ColorBand::Darkest),
            1 => Self::Material(ColorBand::MediumDark),
            2 => Self::Material(ColorBand::MediumLight),
            3 => Self::Material(ColorBand::Lightest),
            Self::BRAND_WHITE_INDEX => Self::BrandWhite,
            _ => unreachable!("the indexed color range is clamped"),
        }
    }
}

#[cfg(test)]
mod color_band_tests {
    use super::*;

    #[test]
    fn from_index_clamped_saturates_at_both_ends() {
        assert_eq!(ColorBand::from_index_clamped(-40), ColorBand::Darkest);
        assert_eq!(ColorBand::from_index_clamped(0), ColorBand::Darkest);
        assert_eq!(ColorBand::from_index_clamped(1), ColorBand::MediumDark);
        assert_eq!(ColorBand::from_index_clamped(2), ColorBand::MediumLight);
        assert_eq!(ColorBand::from_index_clamped(3), ColorBand::Lightest);
        assert_eq!(ColorBand::from_index_clamped(40), ColorBand::Lightest);
    }

    #[test]
    fn from_index_clamped_round_trips_as_index() {
        for band in [
            ColorBand::Darkest,
            ColorBand::MediumDark,
            ColorBand::MediumLight,
            ColorBand::Lightest,
        ] {
            assert_eq!(ColorBand::from_index_clamped(band.as_index() as i32), band);
        }
    }

    #[test]
    fn indexed_color_shifts_through_brand_and_material_bands() {
        use IndexedColor::{BrandBlack, BrandWhite, Material};

        assert_eq!(Material(ColorBand::Darkest).shift(-1), BrandBlack);
        assert_eq!(BrandWhite.shift(-1), Material(ColorBand::Lightest));
        assert_eq!(BrandBlack.shift(1), Material(ColorBand::Darkest));
        assert_eq!(Material(ColorBand::Lightest).shift(1), BrandWhite);
        assert_eq!(BrandBlack.shift(-3), BrandBlack);
        assert_eq!(BrandWhite.shift(3), BrandWhite);
    }

    #[test]
    fn authored_material_palettes_keep_their_requested_bands() {
        assert_eq!(
            CellMaterialId::Wood.resolve_band(ColorBand::Lightest),
            [
                0xea as f32 / 255.0,
                0x98 as f32 / 255.0,
                0x27 as f32 / 255.0,
                1.0
            ]
        );
        assert_eq!(
            CellMaterialId::Stone.resolve_band(ColorBand::MediumDark),
            [
                0x40 as f32 / 255.0,
                0x48 as f32 / 255.0,
                0x63 as f32 / 255.0,
                1.0
            ]
        );
        for material in [CellMaterialId::Mole, CellMaterialId::Dirt] {
            assert_eq!(
                material.resolve_band(ColorBand::MediumLight),
                [
                    0xc4 as f32 / 255.0,
                    0x70 as f32 / 255.0,
                    0x2b as f32 / 255.0,
                    1.0
                ]
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellMaterialId {
    /// Renderer-provided neutral test material retained for consumers that
    /// have not yet authored a palette.
    GrayScale,
    /// Consumer-authored wood palette, carried through the shared renderer
    /// material vocabulary.
    Wood,
    /// Consumer-authored stone palette, carried through the shared renderer
    /// material vocabulary.
    Stone,
    /// Mole's separate palette, even where it temporarily shares dirt's
    /// bands, so either can change independently.
    Mole,
    /// Dirt's separate palette, kept distinct from Mole for future authoring.
    Dirt,
}

impl CellMaterialId {
    pub const fn all() -> &'static [Self] {
        &[
            Self::GrayScale,
            Self::Wood,
            Self::Stone,
            Self::Mole,
            Self::Dirt,
        ]
    }

    /// Resolves one portable material asset filename to its runtime id. This
    /// is the built-in bridge until the renderer asset registry lands.
    pub fn resolve_material_asset(asset_file: &str) -> Option<Self> {
        match asset_file {
            "materials/gray-scale.json" => Some(Self::GrayScale),
            "materials/wood.json" => Some(Self::Wood),
            "materials/stone.json" => Some(Self::Stone),
            "materials/mole.json" => Some(Self::Mole),
            "materials/dirt.json" => Some(Self::Dirt),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::GrayScale => "gray-scale",
            Self::Wood => "wood",
            Self::Stone => "stone",
            Self::Mole => "mole",
            Self::Dirt => "dirt",
        }
    }

    pub fn resolve_band(self, band: ColorBand) -> [f32; 4] {
        match self {
            Self::GrayScale => GRAY_SCALE_BANDS[band.as_index()],
            Self::Wood => WOOD_BANDS[band.as_index()],
            Self::Stone => STONE_BANDS[band.as_index()],
            Self::Mole => MOLE_BANDS[band.as_index()],
            Self::Dirt => DIRT_BANDS[band.as_index()],
        }
    }

    pub fn resolve_glyph_default(self) -> [f32; 4] {
        self.resolve_band(ColorBand::MediumLight)
    }
}

/// The original neutral palette remains a deliberate renderer test material.
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

/// Mole slice authored palette: darkest through lightest wood bands.
const WOOD_BANDS: [[f32; 4]; 4] = [
    hex_rgba(0x3d, 0x23, 0x29),
    hex_rgba(0x81, 0x17, 0x2a),
    hex_rgba(0xa8, 0x56, 0x1a),
    hex_rgba(0xea, 0x98, 0x27),
];

/// Mole slice authored palette: darkest through lightest stone bands.
const STONE_BANDS: [[f32; 4]; 4] = [
    hex_rgba(0x3d, 0x23, 0x29),
    hex_rgba(0x40, 0x48, 0x63),
    hex_rgba(0x78, 0x7d, 0x8b),
    hex_rgba(0xc5, 0xb5, 0xa8),
];

/// Mole palette, independently named even while dirt uses the same values.
const MOLE_BANDS: [[f32; 4]; 4] = [
    hex_rgba(0x3d, 0x23, 0x29),
    hex_rgba(0x89, 0x48, 0x35),
    hex_rgba(0xc4, 0x70, 0x2b),
    hex_rgba(0xf0, 0xac, 0x90),
];

/// Dirt palette, intentionally copied rather than aliased to Mole.
const DIRT_BANDS: [[f32; 4]; 4] = [
    hex_rgba(0x3d, 0x23, 0x29),
    hex_rgba(0x89, 0x48, 0x35),
    hex_rgba(0xc4, 0x70, 0x2b),
    hex_rgba(0xf0, 0xac, 0x90),
];

const fn hex_rgba(red: u8, green: u8, blue: u8) -> [f32; 4] {
    [
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    ]
}
