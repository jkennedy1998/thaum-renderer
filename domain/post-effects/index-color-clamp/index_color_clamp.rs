#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexColorClampEffect {
    pub enabled: bool,
    pub palette: Vec<[u8; 4]>,
}

impl IndexColorClampEffect {
    pub fn new(palette: Vec<[u8; 4]>) -> Self {
        Self {
            enabled: !palette.is_empty(),
            palette,
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled && !self.palette.is_empty()
    }
}

pub fn clamp_rgba_to_index_palette(rgba: [f32; 4], palette: &[[u8; 4]]) -> [f32; 4] {
    if palette.is_empty() {
        return rgba;
    }

    let alpha = rgba[3].clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let rgb_u8 = [
        float_channel_to_u8(rgba[0]),
        float_channel_to_u8(rgba[1]),
        float_channel_to_u8(rgba[2]),
    ];
    let palette_index = nearest_visible_rgb_in_collection(rgb_u8, palette);
    let palette_rgba = palette[palette_index];

    [
        palette_rgba[0] as f32 / 255.0,
        palette_rgba[1] as f32 / 255.0,
        palette_rgba[2] as f32 / 255.0,
        alpha,
    ]
}

pub fn clamp_rgba_collection_to_index_palette(colors: &mut [[f32; 4]], palette: &[[u8; 4]]) {
    if palette.is_empty() {
        return;
    }

    for color in colors {
        *color = clamp_rgba_to_index_palette(*color, palette);
    }
}

fn float_channel_to_u8(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn nearest_visible_rgb_in_collection(input: [u8; 3], collection: &[[u8; 4]]) -> usize {
    let mut best_index = 0;
    let mut best_distance = u32::MAX;

    for (index, candidate) in collection.iter().enumerate() {
        if candidate[3] == 0 {
            continue;
        }

        let distance = squared_rgb_distance(input, [candidate[0], candidate[1], candidate[2]]);
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }

    best_index
}

fn squared_rgb_distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    let red = left[0] as i32 - right[0] as i32;
    let green = left[1] as i32 - right[1] as i32;
    let blue = left[2] as i32 - right[2] as i32;
    (red * red + green * green + blue * blue) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PALETTE: [[u8; 4]; 4] = [
        [0x00, 0x00, 0x00, 0x00],
        [0x87, 0xaa, 0xac, 0x80],
        [0xc7, 0xf2, 0xc7, 0x40],
        [0x10, 0x20, 0x30, 0xff],
    ];

    #[test]
    fn effect_defaults_off_when_no_palette_is_present() {
        assert!(!IndexColorClampEffect::default().is_active());
    }

    #[test]
    fn effect_defaults_on_when_palette_is_present() {
        assert!(IndexColorClampEffect::new(TEST_PALETTE.to_vec()).is_active());
    }

    #[test]
    fn clamp_preserves_exact_palette_rgb_and_original_alpha() {
        let clamped = clamp_rgba_to_index_palette(
            [
                0x87 as f32 / 255.0,
                0xaa as f32 / 255.0,
                0xac as f32 / 255.0,
                0x66 as f32 / 255.0,
            ],
            &TEST_PALETTE,
        );

        assert_eq!(
            clamped,
            [
                0x87 as f32 / 255.0,
                0xaa as f32 / 255.0,
                0xac as f32 / 255.0,
                0x66 as f32 / 255.0
            ]
        );
    }

    #[test]
    fn clamp_moves_non_palette_colors_to_nearest_palette_match_but_preserves_alpha() {
        let clamped = clamp_rgba_to_index_palette([0.54, 0.68, 0.69, 0.52], &TEST_PALETTE);

        assert_eq!(
            clamped,
            [
                0x87 as f32 / 255.0,
                0xaa as f32 / 255.0,
                0xac as f32 / 255.0,
                0.52
            ]
        );
    }

    #[test]
    fn clamp_collection_updates_every_color_in_place() {
        let mut colors = [[0.54, 0.68, 0.69, 0.52], [0.78, 0.95, 0.78, 0.20]];

        clamp_rgba_collection_to_index_palette(&mut colors, &TEST_PALETTE);

        assert_eq!(
            colors,
            [
                [
                    0x87 as f32 / 255.0,
                    0xaa as f32 / 255.0,
                    0xac as f32 / 255.0,
                    0.52
                ],
                [
                    0xc7 as f32 / 255.0,
                    0xf2 as f32 / 255.0,
                    0xc7 as f32 / 255.0,
                    0.20
                ],
            ]
        );
    }

    #[test]
    fn clamp_preserves_fully_transparent_pixels() {
        assert_eq!(
            clamp_rgba_to_index_palette([0.4, 0.5, 0.6, 0.0], &TEST_PALETTE),
            [0.0, 0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn clamp_with_empty_palette_is_a_noop() {
        let original = [0.1, 0.2, 0.3, 0.4];
        assert_eq!(clamp_rgba_to_index_palette(original, &[]), original);
    }
}
