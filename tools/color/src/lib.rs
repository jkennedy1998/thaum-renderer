pub fn nearest_rgb_in_collection(input: [u8; 3], collection: &[[u8; 3]]) -> usize {
    let mut best_index = 0;
    let mut best_distance = u32::MAX;

    for (index, candidate) in collection.iter().enumerate() {
        let distance = squared_rgb_distance(input, *candidate);
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }

    best_index
}

fn squared_rgb_distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    let red = left[0] as i32 - right[0] as i32;
    let green = left[1] as i32 - right[1] as i32;
    let blue = left[2] as i32 - right[2] as i32;
    (red * red + green * green + blue * blue) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rgb_in_collection_picks_exact_match_when_present() {
        let palette = [[0, 0, 0], [127, 127, 127], [255, 255, 255]];
        assert_eq!(nearest_rgb_in_collection([127, 127, 127], &palette), 1);
    }

    #[test]
    fn nearest_rgb_in_collection_is_deterministic_for_non_exact_matches() {
        let palette = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        assert_eq!(nearest_rgb_in_collection([240, 10, 10], &palette), 0);
        assert_eq!(nearest_rgb_in_collection([10, 240, 10], &palette), 1);
        assert_eq!(nearest_rgb_in_collection([10, 10, 240], &palette), 2);
    }
}
