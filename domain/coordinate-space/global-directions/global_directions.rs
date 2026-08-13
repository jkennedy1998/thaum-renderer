#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorldAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisSign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlobalDirection {
    East,
    West,
    Top,
    Bottom,
    South,
    North,
}

impl GlobalDirection {
    pub const fn axis(self) -> WorldAxis {
        match self {
            Self::East | Self::West => WorldAxis::X,
            Self::Top | Self::Bottom => WorldAxis::Y,
            Self::South | Self::North => WorldAxis::Z,
        }
    }

    pub const fn sign(self) -> AxisSign {
        match self {
            Self::East | Self::Top | Self::South => AxisSign::Positive,
            Self::West | Self::Bottom | Self::North => AxisSign::Negative,
        }
    }

    pub const fn unit_vector(self) -> [i32; 3] {
        match self {
            Self::East => [1, 0, 0],
            Self::West => [-1, 0, 0],
            Self::Top => [0, 1, 0],
            Self::Bottom => [0, -1, 0],
            Self::South => [0, 0, 1],
            Self::North => [0, 0, -1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_directions_resolve_to_expected_axes() {
        assert_eq!(GlobalDirection::East.axis(), WorldAxis::X);
        assert_eq!(GlobalDirection::Top.axis(), WorldAxis::Y);
        assert_eq!(GlobalDirection::North.axis(), WorldAxis::Z);
    }

    #[test]
    fn global_directions_resolve_to_expected_signs() {
        assert_eq!(GlobalDirection::East.sign(), AxisSign::Positive);
        assert_eq!(GlobalDirection::Bottom.sign(), AxisSign::Negative);
        assert_eq!(GlobalDirection::South.sign(), AxisSign::Positive);
    }

    #[test]
    fn global_directions_expose_unit_vectors() {
        assert_eq!(GlobalDirection::West.unit_vector(), [-1, 0, 0]);
        assert_eq!(GlobalDirection::Top.unit_vector(), [0, 1, 0]);
        assert_eq!(GlobalDirection::North.unit_vector(), [0, 0, -1]);
    }
}
