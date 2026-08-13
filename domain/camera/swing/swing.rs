#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraSwing {
    PosX,
    NegX,
    PosY,
    NegY,
    #[default]
    PosZ,
    NegZ,
}

impl CameraSwing {
    pub const fn rotate_clockwise(self) -> Self {
        match self {
            Self::PosZ => Self::PosX,
            Self::PosX => Self::NegZ,
            Self::NegZ => Self::NegX,
            Self::NegX => Self::PosZ,
            Self::PosY | Self::NegY => self,
        }
    }

    pub const fn rotate_counter_clockwise(self) -> Self {
        match self {
            Self::PosZ => Self::NegX,
            Self::NegX => Self::NegZ,
            Self::NegZ => Self::PosX,
            Self::PosX => Self::PosZ,
            Self::PosY | Self::NegY => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_swing_is_positive_z() {
        assert_eq!(CameraSwing::default(), CameraSwing::PosZ);
    }

    #[test]
    fn clockwise_rotation_walks_the_authored_horizontal_ring() {
        assert_eq!(CameraSwing::PosZ.rotate_clockwise(), CameraSwing::PosX);
        assert_eq!(CameraSwing::PosX.rotate_clockwise(), CameraSwing::NegZ);
        assert_eq!(CameraSwing::NegZ.rotate_clockwise(), CameraSwing::NegX);
        assert_eq!(CameraSwing::NegX.rotate_clockwise(), CameraSwing::PosZ);
    }

    #[test]
    fn counter_clockwise_rotation_walks_the_authored_horizontal_ring() {
        assert_eq!(
            CameraSwing::PosZ.rotate_counter_clockwise(),
            CameraSwing::NegX
        );
        assert_eq!(
            CameraSwing::NegX.rotate_counter_clockwise(),
            CameraSwing::NegZ
        );
        assert_eq!(
            CameraSwing::NegZ.rotate_counter_clockwise(),
            CameraSwing::PosX
        );
        assert_eq!(
            CameraSwing::PosX.rotate_counter_clockwise(),
            CameraSwing::PosZ
        );
    }

    #[test]
    fn vertical_views_stay_stable_under_horizontal_rotation_helpers() {
        assert_eq!(CameraSwing::PosY.rotate_clockwise(), CameraSwing::PosY);
        assert_eq!(
            CameraSwing::NegY.rotate_counter_clockwise(),
            CameraSwing::NegY
        );
    }
}
