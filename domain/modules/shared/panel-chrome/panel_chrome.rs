use crate::{
    Cell, CellColor, CellGraphic, CellPoint, CellWeight, ModuleRect, UiColorRole, UiPalette,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelBorderEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Box-drawing border weight, matching the old mono_ui `BORDER_STYLES` set
/// (`single`/`double`/`thick`) minus the junction glyphs, which no consumer
/// needs yet (no dividers built here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelBorderStyle {
    Single,
    Double,
    Thick,
}

struct BorderGlyphs {
    corner_tl: char,
    corner_tr: char,
    corner_bl: char,
    corner_br: char,
    horizontal: char,
    vertical: char,
}

impl PanelBorderStyle {
    fn glyphs(self) -> BorderGlyphs {
        match self {
            PanelBorderStyle::Single => BorderGlyphs {
                corner_tl: '┌',
                corner_tr: '┐',
                corner_bl: '└',
                corner_br: '┘',
                horizontal: '─',
                vertical: '│',
            },
            PanelBorderStyle::Double => BorderGlyphs {
                corner_tl: '╔',
                corner_tr: '╗',
                corner_bl: '╚',
                corner_br: '╝',
                horizontal: '═',
                vertical: '║',
            },
            PanelBorderStyle::Thick => BorderGlyphs {
                corner_tl: '┏',
                corner_tr: '┓',
                corner_bl: '┗',
                corner_br: '┛',
                horizontal: '━',
                vertical: '┃',
            },
        }
    }
}

fn chrome_cell(x: i32, y: i32, glyph: char, color: CellColor, weight_index: i32) -> Cell {
    Cell {
        position: CellPoint { x, y, z: 0 },
        graphic: CellGraphic::Glyph(glyph),
        color,
        weight: CellWeight::from_index_clamped(weight_index),
        ..Cell::default()
    }
}

/// A box-drawing border, optional background fill, and optional title —
/// the renderer's port of the old mono_ui `draw_module_border`. Produces
/// `Cell`s positioned local to the panel's own rect (its bottom-left corner
/// is local `(0, 0)`), for a module's `draw()` to merge into its own
/// `CellGroup` alongside its content cells via `CellGroup::extend`.
///
/// Content should stay inset by `content_inset()` from every edge to leave
/// the border and background clear; the title is drawn directly into the
/// top border row rather than reserving an interior row for it, so it never
/// competes with a module's own content for space.
pub struct PanelChrome {
    rect: ModuleRect,
    style: PanelBorderStyle,
    border_color: CellColor,
    left_border_color: Option<CellColor>,
    right_border_color: Option<CellColor>,
    top_border_color: Option<CellColor>,
    bottom_border_color: Option<CellColor>,
    background_color: Option<CellColor>,
    title: Option<String>,
    title_color: CellColor,
    title_start_x: i32,
    border_weight_index: i32,
    left_border_weight_index: Option<i32>,
    right_border_weight_index: Option<i32>,
    top_border_weight_index: Option<i32>,
    bottom_border_weight_index: Option<i32>,
    title_weight_index: i32,
}

impl PanelChrome {
    /// Defaults mirror the old mono_ui `get_standard_ux_chrome_colors()`:
    /// thick border in the palette's `dimmest` role, background fill in
    /// `background`, title text in `medium`, starting two cells in from the
    /// left corner (matching the old default when no gizmos reserve space).
    pub fn new(rect: ModuleRect, palette: &UiPalette) -> Self {
        Self {
            rect,
            style: PanelBorderStyle::Thick,
            border_color: palette.get(UiColorRole::Dimmest),
            left_border_color: None,
            right_border_color: None,
            top_border_color: None,
            bottom_border_color: None,
            background_color: Some(palette.get(UiColorRole::Background)),
            title: None,
            title_color: palette.get(UiColorRole::Medium),
            title_start_x: 2,
            border_weight_index: 2,
            left_border_weight_index: None,
            right_border_weight_index: None,
            top_border_weight_index: None,
            bottom_border_weight_index: None,
            title_weight_index: 2,
        }
    }

    pub fn with_style(mut self, style: PanelBorderStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_border_color(mut self, color: CellColor) -> Self {
        self.border_color = color;
        self
    }

    pub fn with_border_weight(mut self, weight_index: i32) -> Self {
        self.border_weight_index = weight_index;
        self
    }

    pub fn with_edge_color(mut self, edge: PanelBorderEdge, color: CellColor) -> Self {
        match edge {
            PanelBorderEdge::Left => self.left_border_color = Some(color),
            PanelBorderEdge::Right => self.right_border_color = Some(color),
            PanelBorderEdge::Top => self.top_border_color = Some(color),
            PanelBorderEdge::Bottom => self.bottom_border_color = Some(color),
        }
        self
    }

    pub fn with_edge_weight(mut self, edge: PanelBorderEdge, weight_index: i32) -> Self {
        match edge {
            PanelBorderEdge::Left => self.left_border_weight_index = Some(weight_index),
            PanelBorderEdge::Right => self.right_border_weight_index = Some(weight_index),
            PanelBorderEdge::Top => self.top_border_weight_index = Some(weight_index),
            PanelBorderEdge::Bottom => self.bottom_border_weight_index = Some(weight_index),
        }
        self
    }

    pub fn without_background(mut self) -> Self {
        self.background_color = None;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override where the title starts (local columns from the left
    /// corner). A gizmo bar sitting on the same border row calls
    /// `GizmoBar::title_start_x()` and passes it here so the title never
    /// overlaps the gizmo glyphs.
    pub fn with_title_start_x(mut self, start_x: i32) -> Self {
        self.title_start_x = start_x;
        self
    }

    /// How far a module's own content should inset from the left/right and
    /// bottom edges to clear the border.
    pub const fn content_inset() -> i32 {
        1
    }

    /// The top header row inside the border is reserved for gizmos/title,
    /// matching the old system layout.
    pub const fn content_top_inset() -> i32 {
        2
    }

    /// The local bottom-left origin of the usable content area.
    pub fn content_origin() -> (i32, i32) {
        (Self::content_inset(), Self::content_inset())
    }

    /// The `(width, height)` in cells actually available to a module's own
    /// content inside `rect` once the border and header row are cleared.
    pub fn content_size(rect: ModuleRect) -> (i32, i32) {
        let width = (rect.x1 - rect.x0 - Self::content_inset() * 2).max(0);
        let height = (rect.y1 - rect.y0 - Self::content_inset() - Self::content_top_inset()).max(0);
        (width, height)
    }

    /// The absolute rect of the usable content area inside `rect`, after the
    /// border and top gizmo/title row are reserved.
    pub fn content_rect(rect: ModuleRect) -> ModuleRect {
        let (width, height) = Self::content_size(rect);
        let (offset_x, offset_y) = Self::content_origin();
        ModuleRect {
            x0: rect.x0 + offset_x,
            y0: rect.y0 + offset_y,
            x1: rect.x0 + offset_x + width.saturating_sub(1),
            y1: rect.y0 + offset_y + height.saturating_sub(1),
        }
    }

    fn edge_color(&self, edge: PanelBorderEdge) -> CellColor {
        match edge {
            PanelBorderEdge::Left => self.left_border_color.unwrap_or(self.border_color),
            PanelBorderEdge::Right => self.right_border_color.unwrap_or(self.border_color),
            PanelBorderEdge::Top => self.top_border_color.unwrap_or(self.border_color),
            PanelBorderEdge::Bottom => self.bottom_border_color.unwrap_or(self.border_color),
        }
    }

    fn edge_weight_index(&self, edge: PanelBorderEdge) -> i32 {
        match edge {
            PanelBorderEdge::Left => self
                .left_border_weight_index
                .unwrap_or(self.border_weight_index),
            PanelBorderEdge::Right => self
                .right_border_weight_index
                .unwrap_or(self.border_weight_index),
            PanelBorderEdge::Top => self
                .top_border_weight_index
                .unwrap_or(self.border_weight_index),
            PanelBorderEdge::Bottom => self
                .bottom_border_weight_index
                .unwrap_or(self.border_weight_index),
        }
    }

    /// Border, background, and title cells, positioned local to `rect`.
    pub fn cells(&self) -> Vec<Cell> {
        let width = self.rect.x1 - self.rect.x0;
        let height = self.rect.y1 - self.rect.y0;
        let glyphs = self.style.glyphs();
        let mut cells = Vec::new();

        if let Some(background) = self.background_color {
            for x in 1..width {
                for y in 1..height {
                    cells.push(chrome_cell(x, y, ' ', background, 1));
                }
            }
        }

        for x in 0..=width {
            let bottom_glyph = if x == 0 {
                glyphs.corner_bl
            } else if x == width {
                glyphs.corner_br
            } else {
                glyphs.horizontal
            };
            let top_glyph = if x == 0 {
                glyphs.corner_tl
            } else if x == width {
                glyphs.corner_tr
            } else {
                glyphs.horizontal
            };
            cells.push(chrome_cell(
                x,
                0,
                bottom_glyph,
                self.edge_color(PanelBorderEdge::Bottom),
                self.edge_weight_index(PanelBorderEdge::Bottom),
            ));
            cells.push(chrome_cell(
                x,
                height,
                top_glyph,
                self.edge_color(PanelBorderEdge::Top),
                self.edge_weight_index(PanelBorderEdge::Top),
            ));
        }

        for y in 1..height {
            cells.push(chrome_cell(
                0,
                y,
                glyphs.vertical,
                self.edge_color(PanelBorderEdge::Left),
                self.edge_weight_index(PanelBorderEdge::Left),
            ));
            cells.push(chrome_cell(
                width,
                y,
                glyphs.vertical,
                self.edge_color(PanelBorderEdge::Right),
                self.edge_weight_index(PanelBorderEdge::Right),
            ));
        }

        if let Some(title) = &self.title {
            let start_x = self.title_start_x;
            for (index, glyph) in title.chars().enumerate() {
                let x = start_x + index as i32;
                if x >= width {
                    break;
                }
                cells.push(chrome_cell(
                    x,
                    height - 1,
                    glyph,
                    self.title_color,
                    self.title_weight_index,
                ));
            }
        }

        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> ModuleRect {
        ModuleRect { x0, y0, x1, y1 }
    }

    fn cell_at(cells: &[Cell], x: i32, y: i32) -> &Cell {
        cells
            .iter()
            .rev()
            .find(|cell| cell.position == CellPoint { x, y, z: 0 })
            .expect("expected a chrome cell at this position")
    }

    #[test]
    fn draws_all_four_corners_of_the_chosen_style() {
        let chrome = PanelChrome::new(rect(0, 0, 10, 6), &UiPalette::default());
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 0, 0).graphic, CellGraphic::Glyph('┗'));
        assert_eq!(cell_at(&cells, 10, 0).graphic, CellGraphic::Glyph('┛'));
        assert_eq!(cell_at(&cells, 0, 6).graphic, CellGraphic::Glyph('┏'));
        assert_eq!(cell_at(&cells, 10, 6).graphic, CellGraphic::Glyph('┓'));
    }

    #[test]
    fn single_style_uses_single_line_glyphs() {
        let chrome = PanelChrome::new(rect(0, 0, 4, 4), &UiPalette::default())
            .with_style(PanelBorderStyle::Single);
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 0, 0).graphic, CellGraphic::Glyph('└'));
        assert_eq!(cell_at(&cells, 2, 4).graphic, CellGraphic::Glyph('─'));
    }

    #[test]
    fn background_fill_covers_only_the_interior() {
        let chrome = PanelChrome::new(rect(0, 0, 4, 4), &UiPalette::default());
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 2, 2).graphic, CellGraphic::Glyph(' '));
        assert_eq!(
            cell_at(&cells, 2, 2).color,
            UiPalette::default().get(UiColorRole::Background)
        );
    }

    #[test]
    fn without_background_omits_interior_fill_cells() {
        let bordered = PanelChrome::new(rect(0, 0, 4, 4), &UiPalette::default());
        let plain = PanelChrome::new(rect(0, 0, 4, 4), &UiPalette::default()).without_background();

        assert!(bordered.cells().len() > plain.cells().len());
        assert!(plain
            .cells()
            .iter()
            .all(|cell| cell.position != CellPoint { x: 2, y: 2, z: 0 }));
    }

    #[test]
    fn content_size_subtracts_the_border_inset_from_every_side() {
        assert_eq!(PanelChrome::content_size(rect(0, 0, 12, 8)), (10, 5));
        assert_eq!(PanelChrome::content_size(rect(5, 5, 25, 15)), (18, 7));
    }

    #[test]
    fn content_size_never_goes_negative_for_a_too_small_rect() {
        assert_eq!(PanelChrome::content_size(rect(0, 0, 1, 1)), (0, 0));
        assert_eq!(PanelChrome::content_size(rect(0, 0, 2, 2)), (0, 0));
    }

    #[test]
    fn content_rect_matches_the_reserved_border_and_header_area() {
        assert_eq!(
            PanelChrome::content_rect(rect(-4, -2, 8, 6)),
            ModuleRect {
                x0: -3,
                y0: -1,
                x1: 6,
                y1: 3,
            }
        );
    }

    #[test]
    fn title_is_drawn_into_the_header_row_starting_two_cells_in() {
        let chrome = PanelChrome::new(rect(0, 0, 10, 4), &UiPalette::default()).with_title("HI");
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 2, 3).graphic, CellGraphic::Glyph('H'));
        assert_eq!(cell_at(&cells, 3, 3).graphic, CellGraphic::Glyph('I'));
    }

    #[test]
    fn with_title_start_x_moves_the_title_to_clear_reserved_columns() {
        let chrome = PanelChrome::new(rect(0, 0, 10, 4), &UiPalette::default())
            .with_title("HI")
            .with_title_start_x(5);
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 5, 3).graphic, CellGraphic::Glyph('H'));
        assert_eq!(cell_at(&cells, 6, 3).graphic, CellGraphic::Glyph('I'));
    }

    #[test]
    fn title_is_truncated_rather_than_overrunning_the_right_border() {
        let chrome =
            PanelChrome::new(rect(0, 0, 4, 4), &UiPalette::default()).with_title("TOO LONG TITLE");
        let cells = chrome.cells();

        assert!(cells
            .iter()
            .all(|cell| cell.position.x <= 4 && cell.position.x >= 0));
    }

    #[test]
    fn per_edge_color_and_weight_overrides_affect_only_that_edge() {
        let palette = UiPalette::default();
        let chrome = PanelChrome::new(rect(0, 0, 4, 4), &palette)
            .with_border_weight(1)
            .with_edge_color(PanelBorderEdge::Right, palette.get(UiColorRole::Vivid))
            .with_edge_weight(PanelBorderEdge::Right, 3);
        let cells = chrome.cells();

        assert_eq!(cell_at(&cells, 4, 2).color, palette.get(UiColorRole::Vivid));
        assert_eq!(
            cell_at(&cells, 4, 2).weight,
            CellWeight::from_index_clamped(3)
        );
        assert_eq!(
            cell_at(&cells, 0, 2).weight,
            CellWeight::from_index_clamped(1)
        );
    }
}
