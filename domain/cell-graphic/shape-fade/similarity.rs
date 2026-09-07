//! Shape-fade similarity metric (J 2026-09-07): the Dice coefficient over two
//! packed masks — `2|A∩B| / (|A|+|B|)`. Symmetric, one only between identical
//! shapes, zero between disjoint shapes. The empty mask (space) scores zero
//! against every non-empty shape, so a fade whose target is space IS the
//! dissolve; space-to-space is the identity. Shape distance is `1 - dice`,
//! the edge weight the fade-path walk routes over.

use crate::shape_fade::mask_space::PackedMask;

/// Dice similarity in 0..=1. Two empty masks are the same shape (similarity
/// one); an empty and a non-empty mask share nothing (zero).
pub fn dice_similarity(a: &PackedMask, b: &PackedMask) -> f32 {
    let total = a.coverage_count() + b.coverage_count();
    if total == 0 {
        return 1.0;
    }
    let shared = a.intersection_count(b);
    (2.0 * shared as f32) / total as f32
}

/// Shape distance in 0..=1 (`1 - dice`): the edge weight of the fade-path
/// graph. Identical shapes cost nothing; disjoint shapes cost the maximum.
pub fn shape_distance(a: &PackedMask, b: &PackedMask) -> f32 {
    1.0 - dice_similarity(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask_at(indices: &[usize]) -> PackedMask {
        let mut alpha = [0u8; crate::shape_fade::mask_space::MASK_PIXELS];
        for &index in indices {
            alpha[index] = 255;
        }
        PackedMask::from_alpha(&alpha)
    }

    #[test]
    fn identical_masks_score_one() {
        let mask = mask_at(&[0, 5, 30]);
        assert_eq!(dice_similarity(&mask, &mask), 1.0);
        assert_eq!(shape_distance(&mask, &mask), 0.0);
    }

    #[test]
    fn disjoint_masks_score_zero() {
        let a = mask_at(&[0, 1, 2, 3]);
        let b = mask_at(&[100, 101, 102, 103]);
        assert_eq!(dice_similarity(&a, &b), 0.0);
        assert_eq!(shape_distance(&a, &b), 1.0);
    }

    #[test]
    fn the_empty_mask_is_the_dissolve_state() {
        let empty = PackedMask::empty();
        let shape = mask_at(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(dice_similarity(&empty, &shape), 0.0);
        assert_eq!(
            dice_similarity(&empty, &empty),
            1.0,
            "space to space is identity"
        );
    }

    #[test]
    fn partial_overlap_scores_between_the_extremes() {
        let a = mask_at(&[0, 1, 2, 3]);
        let b = mask_at(&[2, 3, 4, 5]);
        let similarity = dice_similarity(&a, &b);
        assert!(similarity > 0.0 && similarity < 1.0, "{similarity}");
        assert_eq!(similarity, 0.5, "2*2 shared / 8 total");
        assert!(shape_distance(&a, &b) < shape_distance(&a, &mask_at(&[50, 51, 52, 53])));
    }
}
