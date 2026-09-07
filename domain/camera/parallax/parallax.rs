use crate::CameraProjectionMode;

use super::perspective::eased_signed_depth_units;

/// User-tunable mouse parallax: when enabled, the live pointer position
/// across the whole screen shifts the view slightly, spread by depth around
/// the focus plane (the focus plane never moves; nearer and farther planes
/// drift in opposite directions).
///
/// `offset` is not a user knob: the host feeds it every frame from the
/// pointer position in clip space (-1..1, x right, y up, `0,0` = screen
/// center) and it is intentionally not persisted — it resets to center.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParallaxProfile {
    /// Whether mouse-driven parallax shifts the view at all.
    pub enabled: bool,
    /// Shift magnitude multiplier. `0.0` = off-feel even when enabled;
    /// the historical-default feel is small on purpose (slight drift).
    pub strength: f32,
    /// Host-fed normalized pointer offset in clip space.
    pub offset: [f32; 2],
}

pub const PARALLAX_DEFAULT_STRENGTH: f32 = 0.15;
pub const PARALLAX_MAX_STRENGTH: f32 = 0.5;

impl Default for ParallaxProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: PARALLAX_DEFAULT_STRENGTH,
            offset: [0.0, 0.0],
        }
    }
}

impl ParallaxProfile {
    /// Upper bound of the strength slider (parallax stays a slight drift).
    pub const MAX_STRENGTH: f32 = PARALLAX_MAX_STRENGTH;
    /// Default strength when the user first enables parallax.
    pub const DEFAULT_STRENGTH: f32 = PARALLAX_DEFAULT_STRENGTH;

    /// Clamp the user-tunable strength into its supported range.
    pub fn with_clamped_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, PARALLAX_MAX_STRENGTH);
        self
    }
}

/// Screen-space (view-plane) shift for one depth unit under the profile.
/// Orthographic never parallaxes; the focus plane (depth 0) never moves.
/// Opposite the pointer sign so nearer planes drift toward the pointer and
/// farther planes drift away, reading as depth rather than as a flat pan.
pub fn parallax_screen_offset(
    profile: ParallaxProfile,
    depth: i32,
    projection_mode: CameraProjectionMode,
) -> [f32; 2] {
    match (profile.enabled, projection_mode) {
        (true, CameraProjectionMode::Perspective) => {
            let magnitude = profile.strength * eased_signed_depth_units(depth);
            [
                -profile.offset[0] * magnitude,
                -profile.offset[1] * magnitude,
            ]
        }
        _ => [0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_profile() -> ParallaxProfile {
        ParallaxProfile {
            enabled: true,
            strength: 0.2,
            offset: [1.0, 0.5],
        }
    }

    #[test]
    fn disabled_or_orthographic_never_shifts() {
        let disabled = ParallaxProfile {
            enabled: false,
            ..enabled_profile()
        };
        assert_eq!(
            parallax_screen_offset(disabled, -4, CameraProjectionMode::Perspective),
            [0.0, 0.0]
        );
        assert_eq!(
            parallax_screen_offset(enabled_profile(), -4, CameraProjectionMode::Orthographic,),
            [0.0, 0.0]
        );
        assert_eq!(
            parallax_screen_offset(
                ParallaxProfile::default(),
                3,
                CameraProjectionMode::Perspective
            ),
            [0.0, 0.0]
        );
    }

    #[test]
    fn focus_plane_never_moves() {
        assert_eq!(
            parallax_screen_offset(enabled_profile(), 0, CameraProjectionMode::Perspective),
            [0.0, 0.0]
        );
    }

    #[test]
    fn near_planes_drift_toward_the_pointer_and_far_planes_away() {
        let near = parallax_screen_offset(enabled_profile(), -4, CameraProjectionMode::Perspective);
        let far = parallax_screen_offset(enabled_profile(), 4, CameraProjectionMode::Perspective);
        // Pointer right (+x): near shifts right, far shifts left.
        assert!(near[0] > 0.0, "near must follow the pointer, got {near:?}");
        assert!(far[0] < 0.0, "far must oppose the pointer, got {far:?}");
        // Pointer up (+y clip space): near shifts up.
        assert!(near[1] > 0.0, "near must follow the pointer, got {near:?}");
        // Both are slight at default-strength scale.
        assert!(near[0].abs() < 2.0 && far[0].abs() < 2.0);
    }

    #[test]
    fn zero_offset_is_a_no_op_even_when_enabled() {
        let centered = ParallaxProfile {
            offset: [0.0, 0.0],
            ..enabled_profile()
        };
        assert_eq!(
            parallax_screen_offset(centered, -6, CameraProjectionMode::Perspective),
            [0.0, 0.0]
        );
    }

    #[test]
    fn strength_clamping_keeps_the_slider_in_range() {
        assert_eq!(
            enabled_profile().with_clamped_strength(9.0).strength,
            PARALLAX_MAX_STRENGTH
        );
        assert_eq!(enabled_profile().with_clamped_strength(-1.0).strength, 0.0);
    }
}
