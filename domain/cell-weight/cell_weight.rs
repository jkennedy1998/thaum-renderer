/// A cell's weight: a plain non-negative index with no engine-enforced
/// ceiling (J 2026-11-05 design pass, project 04n3fq3e). `CellWeight` carries
/// no notion of any typeface's or sprite batch's native weight count — it is
/// not connected to typeface at all, since sprites and other consumers use
/// the same slot. `ZERO`/`ONE`/`TWO`/`THREE` stay as named convenience values
/// for the common first four stops; they are not an exhaustive set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CellWeight(u8);

impl CellWeight {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);
    pub const THREE: Self = Self(3);

    pub const fn as_index(self) -> usize {
        self.0 as usize
    }

    /// Build from a signed index, flooring a negative value at zero. Never
    /// panics; carries no upper bound.
    pub const fn from_index_clamped(index: i32) -> Self {
        if index < 0 {
            Self::ZERO
        } else {
            Self(index as u8)
        }
    }

    /// Clamp into `0..=max_index`: the largest native weight index whatever
    /// is currently loaded (a typeface, a sprite batch) actually supports.
    /// Resolution-time only — never crossfades, snaps to the nearest valid
    /// stop — so a stored weight never panics or crashes when something
    /// with fewer native weights is swapped in.
    pub const fn clamped_to_max(self, max_index: u8) -> Self {
        if self.0 > max_index {
            Self(max_index)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_weight_range_maps_to_stable_indices() {
        assert_eq!(CellWeight::ZERO.as_index(), 0);
        assert_eq!(CellWeight::ONE.as_index(), 1);
        assert_eq!(CellWeight::TWO.as_index(), 2);
        assert_eq!(CellWeight::THREE.as_index(), 3);
    }

    #[test]
    fn negative_indices_floor_at_zero_never_panic() {
        assert_eq!(CellWeight::from_index_clamped(-40), CellWeight::ZERO);
        assert_eq!(CellWeight::from_index_clamped(0), CellWeight::ZERO);
        assert_eq!(CellWeight::from_index_clamped(1), CellWeight::ONE);
    }

    #[test]
    fn from_index_clamped_carries_no_upper_bound() {
        // Unlike the old fixed 4-variant enum, an index beyond today's
        // typefaces' stops is a perfectly valid CellWeight — a typeface with
        // more native weights is free to use it.
        assert_eq!(CellWeight::from_index_clamped(40).as_index(), 40);
    }

    #[test]
    fn clamped_to_max_snaps_down_to_whatever_is_loaded() {
        assert_eq!(CellWeight::from_index_clamped(40).clamped_to_max(3), CellWeight::THREE);
        assert_eq!(CellWeight::ONE.clamped_to_max(3), CellWeight::ONE);
        assert_eq!(CellWeight::ZERO.clamped_to_max(0), CellWeight::ZERO);
    }
}
