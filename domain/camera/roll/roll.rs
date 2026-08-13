#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraRoll {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl CameraRoll {
    pub const fn rotate_clockwise(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    pub const fn rotate_counter_clockwise(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roll_is_zero() {
        assert_eq!(CameraRoll::default(), CameraRoll::Deg0);
    }

    #[test]
    fn clockwise_rotation_walks_the_authored_roll_ring() {
        assert_eq!(CameraRoll::Deg0.rotate_clockwise(), CameraRoll::Deg90);
        assert_eq!(CameraRoll::Deg90.rotate_clockwise(), CameraRoll::Deg180);
        assert_eq!(CameraRoll::Deg180.rotate_clockwise(), CameraRoll::Deg270);
        assert_eq!(CameraRoll::Deg270.rotate_clockwise(), CameraRoll::Deg0);
    }

    #[test]
    fn counter_clockwise_rotation_walks_the_authored_roll_ring() {
        assert_eq!(
            CameraRoll::Deg0.rotate_counter_clockwise(),
            CameraRoll::Deg270
        );
        assert_eq!(
            CameraRoll::Deg270.rotate_counter_clockwise(),
            CameraRoll::Deg180
        );
        assert_eq!(
            CameraRoll::Deg180.rotate_counter_clockwise(),
            CameraRoll::Deg90
        );
        assert_eq!(
            CameraRoll::Deg90.rotate_counter_clockwise(),
            CameraRoll::Deg0
        );
    }
}
