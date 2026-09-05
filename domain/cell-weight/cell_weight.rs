#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CellWeight {
    #[default]
    Zero,
    One,
    Two,
    Three,
}

impl CellWeight {
    pub const fn as_index(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }

    pub const fn from_index_clamped(index: i32) -> Self {
        match index {
            i32::MIN..=0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            _ => Self::Three,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_weight_range_maps_to_stable_indices() {
        assert_eq!(CellWeight::Zero.as_index(), 0);
        assert_eq!(CellWeight::One.as_index(), 1);
        assert_eq!(CellWeight::Two.as_index(), 2);
        assert_eq!(CellWeight::Three.as_index(), 3);
    }

    #[test]
    fn out_of_range_weight_indices_clamp_back_into_the_canonical_range() {
        assert_eq!(CellWeight::from_index_clamped(-40), CellWeight::Zero);
        assert_eq!(CellWeight::from_index_clamped(0), CellWeight::Zero);
        assert_eq!(CellWeight::from_index_clamped(1), CellWeight::One);
        assert_eq!(CellWeight::from_index_clamped(2), CellWeight::Two);
        assert_eq!(CellWeight::from_index_clamped(3), CellWeight::Three);
        assert_eq!(CellWeight::from_index_clamped(40), CellWeight::Three);
    }
}
