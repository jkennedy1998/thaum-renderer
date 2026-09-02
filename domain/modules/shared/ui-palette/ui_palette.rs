use std::{cell::RefCell, rc::Rc};

use crate::CellColor;

/// One semantic color slot any module's chrome can draw from, so a consumer
/// changes its whole UI palette in one place instead of every module
/// hardcoding its own RGB values. Names and default mapping mirror the old
/// mono_ui `ui_customization_store`'s `UiSemanticColorRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiColorRole {
    Background,
    Dimmest,
    Medium,
    Bright,
    Vivid,
    LeftHand,
    RightHand,
}

impl UiColorRole {
    pub const ALL: [UiColorRole; 7] = [
        UiColorRole::Background,
        UiColorRole::Dimmest,
        UiColorRole::Medium,
        UiColorRole::Bright,
        UiColorRole::Vivid,
        UiColorRole::LeftHand,
        UiColorRole::RightHand,
    ];

    fn index(self) -> usize {
        match self {
            UiColorRole::Background => 0,
            UiColorRole::Dimmest => 1,
            UiColorRole::Medium => 2,
            UiColorRole::Bright => 3,
            UiColorRole::Vivid => 4,
            UiColorRole::LeftHand => 5,
            UiColorRole::RightHand => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            UiColorRole::Background => "BACKGROUND",
            UiColorRole::Dimmest => "DIMMEST",
            UiColorRole::Medium => "MEDIUM",
            UiColorRole::Bright => "BRIGHT",
            UiColorRole::Vivid => "VIVID",
            UiColorRole::LeftHand => "LEFT HAND",
            UiColorRole::RightHand => "RIGHT HAND",
        }
    }
}

/// A settable color slot for every `UiColorRole`. Ships with the same
/// role -> color defaults the old system shipped, so existing intuition
/// about "vivid" or "left hand" carries over.
#[derive(Debug, Clone)]
pub struct UiPalette {
    colors: Rc<RefCell<[CellColor; 7]>>,
}

fn flat(rgb: [u8; 3]) -> CellColor {
    CellColor::Flat([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    ])
}

fn rgb_from_color(color: CellColor) -> [u8; 3] {
    let rgba = color.resolve_glyph();
    [
        (rgba[0] * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgba[1] * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgba[2] * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

impl Default for UiPalette {
    fn default() -> Self {
        Self {
            colors: Rc::new(RefCell::new([
                flat([0x12, 0x0a, 0x1a]), // background - off_black
                flat([0x2a, 0x2a, 0x41]), // dimmest - deep_blue
                flat([0x78, 0x7d, 0x8b]), // medium - medium_gray
                flat([0xe0, 0xe8, 0xd0]), // bright - pale_gray
                flat([0x8b, 0xf5, 0xc6]), // vivid - vivid_cyan
                flat([0x27, 0x49, 0xd0]), // left_hand - vivid_blue
                flat([0xdc, 0x34, 0x26]), // right_hand - vivid_red
            ])),
        }
    }
}

impl PartialEq for UiPalette {
    fn eq(&self, other: &Self) -> bool {
        *self.colors.borrow() == *other.colors.borrow()
    }
}

impl UiPalette {
    pub fn get(&self, role: UiColorRole) -> CellColor {
        self.colors.borrow()[role.index()]
    }

    pub fn get_rgb(&self, role: UiColorRole) -> [u8; 3] {
        rgb_from_color(self.get(role))
    }

    pub fn set(&self, role: UiColorRole, color: CellColor) {
        self.colors.borrow_mut()[role.index()] = color;
    }

    pub fn set_rgb(&self, role: UiColorRole, rgb: [u8; 3]) {
        self.set(role, flat(rgb));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_gives_every_role_a_distinct_color() {
        let palette = UiPalette::default();
        let colors: Vec<CellColor> = UiColorRole::ALL
            .iter()
            .map(|role| palette.get(*role))
            .collect();

        for (index, color) in colors.iter().enumerate() {
            for other in &colors[index + 1..] {
                assert_ne!(color, other);
            }
        }
    }

    #[test]
    fn set_overrides_only_the_targeted_role() {
        let palette = UiPalette::default();
        let custom = CellColor::Flat([1.0, 0.0, 0.0, 1.0]);
        palette.set(UiColorRole::Vivid, custom);

        assert_eq!(palette.get(UiColorRole::Vivid), custom);
        assert_ne!(palette.get(UiColorRole::Background), custom);
    }

    #[test]
    fn cloned_palettes_share_the_same_live_state() {
        let palette = UiPalette::default();
        let clone = palette.clone();

        clone.set_rgb(UiColorRole::Bright, [1, 2, 3]);

        assert_eq!(palette.get_rgb(UiColorRole::Bright), [1, 2, 3]);
    }

    #[test]
    fn role_labels_match_the_old_names() {
        assert_eq!(UiColorRole::LeftHand.label(), "LEFT HAND");
        assert_eq!(UiColorRole::Vivid.label(), "VIVID");
    }
}
