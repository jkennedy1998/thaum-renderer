use thaum_renderer_tools_color::nearest_rgb_in_collection;

use crate::ColorBand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteColorChannel {
    A,
    B,
    C,
    AB,
    BC,
    CA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedSpritePixel {
    pub alpha: u8,
    pub channel: SpriteColorChannel,
    pub band: ColorBand,
}

pub fn decode_sprite_rgba(rgba: &[u8]) -> Result<Vec<Option<DecodedSpritePixel>>, String> {
    if rgba.len() % 4 != 0 {
        return Err(format!(
            "sprite rgba buffer length must be divisible by 4, got {}",
            rgba.len()
        ));
    }

    Ok(rgba
        .chunks_exact(4)
        .map(|pixel| decode_sprite_pixel([pixel[0], pixel[1], pixel[2], pixel[3]]))
        .collect())
}

pub fn decode_sprite_pixel(pixel: [u8; 4]) -> Option<DecodedSpritePixel> {
    let [red, green, blue, alpha] = pixel;
    if alpha == 0 || [red, green, blue] == [0, 0, 0] {
        return None;
    }

    if red == green && green == blue {
        return Some(DecodedSpritePixel {
            alpha,
            channel: SpriteColorChannel::A,
            band: ColorBand::MediumLight,
        });
    }

    let palette_index = nearest_rgb_in_collection([red, green, blue], &CANONICAL_SPRITE_PALETTE);
    Some(DecodedSpritePixel {
        alpha,
        channel: palette_channel(palette_index),
        band: palette_band(palette_index),
    })
}

pub const fn canonical_sprite_palette() -> &'static [[u8; 3]; 24] {
    &CANONICAL_SPRITE_PALETTE
}

const CANONICAL_SPRITE_PALETTE: [[u8; 3]; 24] = [
    [0x4a, 0x28, 0x28],
    [0x85, 0x47, 0x47],
    [0xdf, 0x79, 0x79],
    [0xf6, 0xda, 0xda],
    [0x25, 0x3e, 0x25],
    [0x3d, 0x6e, 0x3d],
    [0x6f, 0xb6, 0x6f],
    [0xc7, 0xf2, 0xc7],
    [0x35, 0x35, 0x55],
    [0x5d, 0x5d, 0x94],
    [0x9e, 0x9e, 0xe8],
    [0xdf, 0xdf, 0xf8],
    [0x38, 0x33, 0x27],
    [0x61, 0x5a, 0x42],
    [0xa7, 0x97, 0x74],
    [0xdf, 0xe6, 0xd1],
    [0x2d, 0x39, 0x3d],
    [0x4d, 0x65, 0x69],
    [0x87, 0xaa, 0xac],
    [0xd3, 0xe8, 0xe0],
    [0x3f, 0x2f, 0x3f],
    [0x71, 0x52, 0x6e],
    [0xbe, 0x8c, 0xb1],
    [0xea, 0xdd, 0xe9],
];

const fn palette_channel(index: usize) -> SpriteColorChannel {
    match index / 4 {
        0 => SpriteColorChannel::A,
        1 => SpriteColorChannel::B,
        2 => SpriteColorChannel::C,
        3 => SpriteColorChannel::AB,
        4 => SpriteColorChannel::BC,
        5 => SpriteColorChannel::CA,
        _ => unreachable!(),
    }
}

const fn palette_band(index: usize) -> ColorBand {
    match index % 4 {
        0 => ColorBand::Darkest,
        1 => ColorBand::MediumDark,
        2 => ColorBand::MediumLight,
        3 => ColorBand::Lightest,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_palette_has_twenty_four_colors() {
        assert_eq!(canonical_sprite_palette().len(), 24);
    }

    #[test]
    fn exact_palette_colors_decode_to_expected_channel_and_band() {
        let decoded = decode_sprite_pixel([0x87, 0xaa, 0xac, 0xff]).unwrap();
        assert_eq!(decoded.channel, SpriteColorChannel::BC);
        assert_eq!(decoded.band, ColorBand::MediumLight);
    }

    #[test]
    fn grayscale_pixels_follow_the_medium_light_slot_a_fallback() {
        let decoded = decode_sprite_pixel([0x80, 0x80, 0x80, 0xff]).unwrap();
        assert_eq!(decoded.channel, SpriteColorChannel::A);
        assert_eq!(decoded.band, ColorBand::MediumLight);
    }

    #[test]
    fn transparent_and_black_pixels_decode_as_empty() {
        assert_eq!(decode_sprite_pixel([0, 0, 0, 0]), None);
        assert_eq!(decode_sprite_pixel([0, 0, 0, 0xff]), None);
    }

    #[test]
    fn nearby_noncanonical_colors_match_to_the_nearest_palette_color_without_error() {
        let decoded = decode_sprite_pixel([0x89, 0xad, 0xaf, 0xff]).unwrap();
        assert_eq!(decoded.channel, SpriteColorChannel::BC);
        assert_eq!(decoded.band, ColorBand::MediumLight);
    }
}
