//! Smooth display interpolation between the renderer's authored 24-state
//! camera frames. Semantic swing/roll stay discrete; this seam only supplies
//! the temporary render residual while a one-step transition is in flight.

use super::{CameraRoll, CameraSwing};

pub const CAMERA_TRANSITION_SECONDS: f32 = 0.28;
const HALF_STEP_RADIANS: f32 = std::f32::consts::FRAC_PI_4;

/// A one-step command in the authored six-swing, four-roll camera graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPresentationAction {
    SwingLeft,
    SwingRight,
    SwingUp,
    SwingDown,
    RollCounterClockwise,
    RollClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionPhase {
    BeforeSemanticSnap,
    AfterSemanticSnap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CameraTransition {
    action: CameraPresentationAction,
    source_swing: CameraSwing,
    source_roll: CameraRoll,
    phase: TransitionPhase,
    elapsed_seconds: f32,
}

/// Render-only residual for a single authored-frame transition. The semantic
/// frame changes at the visual midpoint, letting the outgoing and incoming
/// discrete poses join at the same 45-degree presentation angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPresentation {
    transition: Option<CameraTransition>,
    residual_basis: [[f32; 3]; 3],
}

impl Default for CameraPresentation {
    fn default() -> Self {
        Self {
            transition: None,
            residual_basis: identity(),
        }
    }
}

impl CameraPresentation {
    /// Begin one interpolated semantic-frame command. An active transition is
    /// intentionally not retargeted: each press completes one readable step.
    pub fn begin(
        &mut self,
        swing: CameraSwing,
        roll: CameraRoll,
        action: CameraPresentationAction,
    ) -> bool {
        if self.transition.is_some() {
            return false;
        }
        self.transition = Some(CameraTransition {
            action,
            source_swing: swing,
            source_roll: roll,
            phase: TransitionPhase::BeforeSemanticSnap,
            elapsed_seconds: 0.0,
        });
        true
    }

    /// Advance the visual interpolation. Returns the one semantic action to
    /// commit exactly at the midpoint; callers own semantic swing/roll state.
    pub fn advance(
        &mut self,
        swing: CameraSwing,
        roll: CameraRoll,
        delta_seconds: f32,
    ) -> Option<CameraPresentationAction> {
        let Some(mut transition) = self.transition else {
            self.residual_basis = identity();
            return None;
        };
        if (transition.source_swing, transition.source_roll) != (swing, roll) {
            self.reset();
            return None;
        }

        transition.elapsed_seconds += delta_seconds.max(0.0);
        match transition.phase {
            TransitionPhase::BeforeSemanticSnap => {
                let progress = (transition.elapsed_seconds / CAMERA_TRANSITION_SECONDS).min(1.0);
                self.residual_basis = basis_for_tilt(transition.action, ease_in_cubic(progress));
                if progress < 1.0 {
                    self.transition = Some(transition);
                    return None;
                }

                transition.phase = TransitionPhase::AfterSemanticSnap;
                transition.elapsed_seconds = 0.0;
                self.residual_basis = basis_for_tilt(transition.action, -1.0);
                self.transition = Some(transition);
                Some(transition.action)
            }
            TransitionPhase::AfterSemanticSnap => {
                let progress = (transition.elapsed_seconds / CAMERA_TRANSITION_SECONDS).min(1.0);
                self.residual_basis =
                    basis_for_tilt(transition.action, -ease_out_cubic_inverse(progress));
                if progress >= 1.0 {
                    self.reset();
                } else {
                    self.transition = Some(transition);
                }
                None
            }
        }
    }

    /// A midpoint commit changes semantic state. Keep the post-snap half only
    /// when that commit produced the expected authored target frame.
    pub fn accept_semantic_commit(&mut self, swing: CameraSwing, roll: CameraRoll) {
        if let Some(transition) = &mut self.transition {
            transition.source_swing = swing;
            transition.source_roll = roll;
        }
    }

    pub fn residual_basis(&self) -> [[f32; 3]; 3] {
        self.residual_basis
    }

    pub fn reset(&mut self) {
        self.transition = None;
        self.residual_basis = identity();
    }
}

fn ease_in_cubic(progress: f32) -> f32 {
    progress * progress * progress
}

/// Starts at one and eases to zero, matching the old post-snap handoff.
fn ease_out_cubic_inverse(progress: f32) -> f32 {
    let inverse = 1.0 - progress;
    inverse * inverse * inverse
}

fn basis_for_tilt(action: CameraPresentationAction, magnitude: f32) -> [[f32; 3]; 3] {
    let angle = HALF_STEP_RADIANS * magnitude;
    match action {
        // As with roll, these are display-space residuals, whose handedness
        // is opposite the authored semantic swing ring. This changes no hard
        // midpoint commit, control direction, or variant-facing meaning.
        CameraPresentationAction::SwingLeft => rotation_y(-angle),
        CameraPresentationAction::SwingRight => rotation_y(angle),
        CameraPresentationAction::SwingUp => rotation_x(angle),
        CameraPresentationAction::SwingDown => rotation_x(-angle),
        // The semantic roll ring is already correct. Its render residual
        // uses the opposite screen-space handedness, so only this temporary
        // smooth portion is intentionally inverted.
        CameraPresentationAction::RollCounterClockwise => rotation_z(angle),
        CameraPresentationAction::RollClockwise => rotation_z(-angle),
    }
}

fn identity() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn rotation_x(angle: f32) -> [[f32; 3]; 3] {
    let (sine, cosine) = angle.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, cosine, sine], [0.0, -sine, cosine]]
}

fn rotation_y(angle: f32) -> [[f32; 3]; 3] {
    let (sine, cosine) = angle.sin_cos();
    [[cosine, 0.0, -sine], [0.0, 1.0, 0.0], [sine, 0.0, cosine]]
}

fn rotation_z(angle: f32) -> [[f32; 3]; 3] {
    let (sine, cosine) = angle.sin_cos();
    [[cosine, sine, 0.0], [-sine, cosine, 0.0], [0.0, 0.0, 1.0]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_commits_only_at_the_visual_midpoint() {
        let mut presentation = CameraPresentation::default();
        assert!(presentation.begin(
            CameraSwing::PosZ,
            CameraRoll::Deg0,
            CameraPresentationAction::SwingLeft,
        ));
        assert_eq!(
            presentation.advance(
                CameraSwing::PosZ,
                CameraRoll::Deg0,
                CAMERA_TRANSITION_SECONDS
            ),
            Some(CameraPresentationAction::SwingLeft)
        );
        assert_ne!(presentation.residual_basis(), identity());
    }

    #[test]
    fn post_snap_residual_settles_back_to_identity() {
        let mut presentation = CameraPresentation::default();
        presentation.begin(
            CameraSwing::PosZ,
            CameraRoll::Deg0,
            CameraPresentationAction::RollClockwise,
        );
        presentation.advance(
            CameraSwing::PosZ,
            CameraRoll::Deg0,
            CAMERA_TRANSITION_SECONDS,
        );
        presentation.accept_semantic_commit(CameraSwing::PosZ, CameraRoll::Deg90);
        presentation.advance(
            CameraSwing::PosZ,
            CameraRoll::Deg90,
            CAMERA_TRANSITION_SECONDS,
        );

        assert_eq!(presentation.residual_basis(), identity());
    }

    #[test]
    fn residuals_use_the_opposite_screen_space_handedness() {
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::RollClockwise, 1.0),
            rotation_z(-HALF_STEP_RADIANS)
        );
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::RollCounterClockwise, 1.0),
            rotation_z(HALF_STEP_RADIANS)
        );
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::SwingLeft, 1.0),
            rotation_y(-HALF_STEP_RADIANS)
        );
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::SwingRight, 1.0),
            rotation_y(HALF_STEP_RADIANS)
        );
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::SwingUp, 1.0),
            rotation_x(HALF_STEP_RADIANS)
        );
        assert_eq!(
            basis_for_tilt(CameraPresentationAction::SwingDown, 1.0),
            rotation_x(-HALF_STEP_RADIANS)
        );
    }

    #[test]
    fn active_transition_refuses_a_second_retarget_command() {
        let mut presentation = CameraPresentation::default();
        assert!(presentation.begin(
            CameraSwing::PosZ,
            CameraRoll::Deg0,
            CameraPresentationAction::SwingLeft,
        ));
        assert!(!presentation.begin(
            CameraSwing::PosZ,
            CameraRoll::Deg0,
            CameraPresentationAction::SwingRight,
        ));
    }
}
