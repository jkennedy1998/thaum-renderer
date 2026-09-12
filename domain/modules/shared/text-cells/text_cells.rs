//! Shared glyph-run text pushing: the one helper for laying a run of
//! character glyphs into a cell list, column-clamped to a right edge, with
//! an explicit [`CellWeight`].
//!
//! Every module re-rolled this loop — `chars().enumerate()`, an inclusive
//! `max_x` break, and a `Cell { graphic: Glyph(ch), .. }` push — and each
//! copy drifted (most silently dropped the weight, leaving text at the
//! invisible weight-0 chrome). This seam makes the glyph run a shared
//! primitive: color, weight, and the column clamp are explicit arguments,
//! and the return value reports how many glyphs actually landed so callers
//! can measure truncation without re-counting.
//!
//! The helper pushes into a `Vec<Cell>` and draws nothing itself; hit-testing,
//! hotspots, and tooltips stay the owning module's business, matching the
//! text-entry seam's "pure state, no rect" rule.

use crate::{Cell, CellColor, CellGraphic, CellPoint, CellWeight};

/// Push one left-to-right glyph run starting at `start_x` on row `y`,
/// stopping after the inclusive `max_x` column (pass `i32::MAX` for no
/// clamp). Returns the number of glyphs pushed.
pub fn push_text_cells(
    cells: &mut Vec<Cell>,
    start_x: i32,
    y: i32,
    text: &str,
    color: CellColor,
    weight: CellWeight,
    max_x: i32,
) -> usize {
    let mut pushed = 0;
    for (column, glyph) in text.chars().enumerate() {
        let x = start_x + column as i32;
        if x > max_x {
            break;
        }
        cells.push(Cell {
            position: CellPoint { x, y, z: 0 },
            graphic: CellGraphic::Glyph(glyph),
            color,
            weight,
            ..Cell::default()
        });
        pushed += 1;
    }
    pushed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_weight::CellWeight;
    use crate::ui_palette::{UiColorRole, UiPalette};

    fn cell_at(cells: &[Cell], x: i32, y: i32) -> Option<&Cell> {
        cells
            .iter()
            .find(|cell| cell.position.x == x && cell.position.y == y)
    }

    #[test]
    fn pushes_one_glyph_cell_per_character_with_the_given_color_and_weight() {
        let palette = UiPalette::default();
        let mut cells = Vec::new();
        let pushed = push_text_cells(
            &mut cells,
            5,
            3,
            "LAYERS",
            palette.get(UiColorRole::Medium),
            CellWeight::from_index_clamped(1),
            i32::MAX,
        );
        assert_eq!(pushed, 6);
        let first = cell_at(&cells, 5, 3).unwrap();
        assert_eq!(first.graphic, CellGraphic::Glyph('L'));
        assert_eq!(first.weight, CellWeight::from_index_clamped(1));
        assert_eq!(first.color, palette.get(UiColorRole::Medium));
        let last = cell_at(&cells, 10, 3).unwrap();
        assert_eq!(last.graphic, CellGraphic::Glyph('S'));
    }

    #[test]
    fn stops_after_the_inclusive_max_x_column_and_reports_the_truncation() {
        let palette = UiPalette::default();
        let mut cells = Vec::new();
        // Columns 5..=7 inclusive: three glyphs of the five-character text.
        let pushed = push_text_cells(
            &mut cells,
            5,
            2,
            "LABEL",
            palette.get(UiColorRole::Bright),
            CellWeight::from_index_clamped(2),
            7,
        );
        assert_eq!(pushed, 3);
        assert!(cell_at(&cells, 7, 2).is_some());
        assert!(cell_at(&cells, 8, 2).is_none());
    }

    #[test]
    fn weight_and_color_are_explicit_not_silently_defaulted() {
        let palette = UiPalette::default();
        let mut cells = Vec::new();
        push_text_cells(
            &mut cells,
            0,
            0,
            "ab",
            palette.get(UiColorRole::Vivid),
            CellWeight::from_index_clamped(2),
            i32::MAX,
        );
        assert!(cells.iter().all(|cell| {
            cell.weight == CellWeight::from_index_clamped(2)
                && cell.color == palette.get(UiColorRole::Vivid)
        }));
    }
}
