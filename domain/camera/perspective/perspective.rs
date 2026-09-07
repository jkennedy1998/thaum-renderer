use crate::CameraProjectionMode;

/// Focal distance in cells: the depth at which glyphs render at exactly 1x.
/// Smaller values make the perspective feel wider/more extreme, larger
/// values flatter, but it is camera-internal tuning, not a user knob yet.
const PERSPECTIVE_FOCAL_DISTANCE_CELLS: f32 = 13.0;

/// Sublinear easing on raw depth units so the first steps away from the
/// focus plane read strongly before far depths compress together.
const PERSPECTIVE_DEPTH_EASE_POWER: f32 = 0.9;

/// User-tunable perspective shaping: separate multipliers for how depth
/// affects glyph scaling and how depth affects screen-position spread, plus
/// the near-camera floor that stops glyphs from growing without bound.
///
/// The default reproduces the historical single shared factor exactly
/// (both strengths 0.9, floor fraction 0.28). `0.0` strengths give a fully
/// straight-on look (everything at 1x, no positional convergence) while
/// values above the default push more extreme than the old renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveProfile {
    /// Multiplier on the depth effect for glyph scaling. `0.0` = flat
    /// (every glyph 1x regardless of depth), `1.0` = the historical
    /// scale feel, above that increasingly extreme.
    pub scale_strength: f32,
    /// Multiplier on the depth effect for screen-position spread (how much
    /// glyphs converge toward the focus point as they recede and diverge as
    /// they approach). `0.0` = no positional perspective at all.
    pub position_strength: f32,
    /// Projection denominator floor as a fraction of the focal distance.
    /// This is the near-camera saturation: at the default 0.28 glyphs stop
    /// growing past ~3.6x. `0.0` removes the floor so near glyphs keep
    /// growing without bound.
    pub near_floor_fraction: f32,
}

impl Default for PerspectiveProfile {
    fn default() -> Self {
        Self {
            scale_strength: PERSPECTIVE_DEFAULT_STRENGTH,
            position_strength: PERSPECTIVE_DEFAULT_STRENGTH,
            near_floor_fraction: PERSPECTIVE_DEFAULT_NEAR_FLOOR_FRACTION,
        }
    }
}

const PERSPECTIVE_DEFAULT_STRENGTH: f32 = 0.9;
const PERSPECTIVE_DEFAULT_NEAR_FLOOR_FRACTION: f32 = 0.28;

/// Sublinear easing on signed depth units (view-relative cells). The ease
/// power is renderer-internal tuning; user knobs multiply its result.
pub fn eased_signed_depth_units(depth: i32) -> f32 {
    if depth == 0 {
        return 0.0;
    }

    let sign = depth.signum() as f32;
    let magnitude = ((depth.abs() as f32) + 1.0).powf(PERSPECTIVE_DEPTH_EASE_POWER) - 1.0;
    sign * magnitude
}

fn perspective_denominator(depth_units: f32, profile: PerspectiveProfile) -> f32 {
    let floor = if profile.near_floor_fraction > 0.0 {
        PERSPECTIVE_FOCAL_DISTANCE_CELLS * profile.near_floor_fraction
    } else {
        // No user floor: keep only a positive epsilon so the projection can
        // grow without bound but never divide by exactly zero.
        f32::EPSILON
    };
    (PERSPECTIVE_FOCAL_DISTANCE_CELLS + depth_units).max(floor)
}

/// Glyph scale factor for one depth unit under a profile. Orthographic is
/// always 1.0; perspective is focal / (focal + scale_strength * eased(depth)).
pub fn depth_scale_factor(
    depth: i32,
    projection_mode: CameraProjectionMode,
    profile: PerspectiveProfile,
) -> f32 {
    match projection_mode {
        CameraProjectionMode::Perspective => {
            let depth_units = eased_signed_depth_units(depth) * profile.scale_strength;
            PERSPECTIVE_FOCAL_DISTANCE_CELLS / perspective_denominator(depth_units, profile)
        }
        CameraProjectionMode::Orthographic => 1.0,
    }
}

/// Screen-position spread for one depth unit under a profile: the factor
/// that projected (right, up) offsets are multiplied by. Orthographic is
/// always 1.0 (no positional convergence).
pub fn depth_position_spread(
    depth: i32,
    projection_mode: CameraProjectionMode,
    profile: PerspectiveProfile,
) -> f32 {
    match projection_mode {
        CameraProjectionMode::Perspective => {
            let depth_units = eased_signed_depth_units(depth) * profile.position_strength;
            PERSPECTIVE_FOCAL_DISTANCE_CELLS / perspective_denominator(depth_units, profile)
        }
        CameraProjectionMode::Orthographic => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn default_profile_reproduces_the_historical_shared_factor() {
        // Historical: eased(5) = 6^0.9 - 1, then 13 / (13 + 0.9 * eased).
        let eased = 6.0f32.powf(0.9) - 1.0;
        let historical = 13.0 / (13.0 + 0.9 * eased);
        let scale = depth_scale_factor(
            5,
            CameraProjectionMode::Perspective,
            PerspectiveProfile::default(),
        );
        let spread = depth_position_spread(
            5,
            CameraProjectionMode::Perspective,
            PerspectiveProfile::default(),
        );
        assert!(approx(scale, historical), "{scale} vs {historical}");
        assert!(approx(spread, historical), "{spread} vs {historical}");
    }

    #[test]
    fn focus_depth_is_always_one() {
        let profile = PerspectiveProfile {
            scale_strength: 1.8,
            position_strength: 1.5,
            near_floor_fraction: 0.0,
        };
        assert!(approx(
            depth_scale_factor(0, CameraProjectionMode::Perspective, profile),
            1.0
        ));
        assert!(approx(
            depth_position_spread(0, CameraProjectionMode::Perspective, profile),
            1.0
        ));
    }

    #[test]
    fn zero_strengths_give_a_fully_straight_on_look() {
        let profile = PerspectiveProfile {
            scale_strength: 0.0,
            position_strength: 0.0,
            near_floor_fraction: 0.28,
        };
        for depth in [-4, -1, 1, 7] {
            assert!(approx(
                depth_scale_factor(depth, CameraProjectionMode::Perspective, profile),
                1.0
            ));
            assert!(approx(
                depth_position_spread(depth, CameraProjectionMode::Perspective, profile),
                1.0
            ));
        }
    }

    #[test]
    fn orthographic_ignores_the_profile() {
        let profile = PerspectiveProfile {
            scale_strength: 2.0,
            position_strength: 2.0,
            near_floor_fraction: 0.0,
        };
        assert!(approx(
            depth_scale_factor(-3, CameraProjectionMode::Orthographic, profile),
            1.0
        ));
        assert!(approx(
            depth_position_spread(-3, CameraProjectionMode::Orthographic, profile),
            1.0
        ));
    }

    #[test]
    fn near_camera_saturates_at_the_floor_fraction() {
        let profile = PerspectiveProfile::default();
        let ceiling = 1.0 / 0.28;
        let deep = depth_scale_factor(-40, CameraProjectionMode::Perspective, profile);
        let deeper = depth_scale_factor(-400, CameraProjectionMode::Perspective, profile);
        assert!(approx(deep, ceiling));
        assert!(approx(deeper, ceiling));
    }

    #[test]
    fn removing_the_floor_lets_near_glyphs_keep_growing() {
        let floored = PerspectiveProfile::default();
        let unbounded = PerspectiveProfile {
            near_floor_fraction: 0.0,
            ..floored
        };
        let depth = -40;
        let unbounded_scale =
            depth_scale_factor(depth, CameraProjectionMode::Perspective, unbounded);
        assert!(unbounded_scale > 1.0 / 0.28);
        assert!(
            unbounded_scale > depth_scale_factor(depth, CameraProjectionMode::Perspective, floored)
        );
    }

    #[test]
    fn strengths_above_default_push_more_extreme_than_the_old_renderer() {
        let default = PerspectiveProfile::default();
        let extreme = PerspectiveProfile {
            scale_strength: 1.8,
            position_strength: 1.8,
            ..default
        };
        let depth = -3;
        let default_scale = depth_scale_factor(depth, CameraProjectionMode::Perspective, default);
        let extreme_scale = depth_scale_factor(depth, CameraProjectionMode::Perspective, extreme);
        assert!(extreme_scale > default_scale);
        let default_spread =
            depth_position_spread(depth, CameraProjectionMode::Perspective, default);
        let extreme_spread =
            depth_position_spread(depth, CameraProjectionMode::Perspective, extreme);
        assert!(extreme_spread > default_spread);
    }
}
