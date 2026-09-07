use std::time::Duration;

use crate::{
    Cell, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight, ModuleRect,
    UiColorRole, UiPalette, WorldPoint,
};

/// How long the pointer must rest on one hotspot before its card appears.
/// Middle of the 300–500ms hover-delay consensus from tooltip UX research.
pub const DWELL: Duration = Duration::from_millis(400);

/// Longest single tooltip text line in cells; longer text wraps into
/// multiple centered lines.
pub const TEXT_WRAP_COLUMNS: usize = 35;

/// Depth of the card's overlay layer (ring, corners, text) above the solid
/// backer at z 0, inside the same Flat2d group. Real depth, not overwrite:
/// the backer stays valid behind every overlay cell, and under the camera's
/// perspective the overlay picks up the Flat2d local-depth offset, so the
/// card participates in parallax instead of fighting it.
pub const TEXT_DEPTH: i32 = 1;

/// One hoverable, explainable piece of UI: its screen-space rect plus the
/// two-line card copy (what it is, how to interact with it). Modules
/// declare hotspots; the shared tooltip state and card renderer do
/// everything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotspot {
    /// Absolute screen-space rect in the module-coordinate space (the same
    /// space module rects and pointer events live in). May be a single cell.
    pub rect: ModuleRect,
    pub title: String,
    pub description: String,
}

impl Hotspot {
    pub fn new(rect: ModuleRect, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            rect,
            title: title.into(),
            description: description.into(),
        }
    }
}

/// Shared hover-dwell state driving one tooltip card at a time. The host
/// feeds it each frame with the hotspot currently under the pointer (if
/// any) and the elapsed frame time; it answers with the card to draw, if
/// the dwell has elapsed, and whether the frame loop must keep running.
///
/// Suppression policy lives with the caller: while a pointer button is
/// held (or a module holds pointer capture) the host passes `None`, so a
/// press never fires or keeps a card alive.
#[derive(Debug, Default)]
pub struct TooltipState {
    dwell: Duration,
    subject: Option<Hotspot>,
    active: bool,
}

/// One frame's tooltip outcome. `card` is what the host should render this
/// frame; `dirty` is whether the tooltip needs more frames soon (dwell
/// running, or the card's visibility just changed), so a demand-driven
/// frame loop knows not to idle.
#[derive(Debug, PartialEq, Eq)]
pub struct TooltipFrame {
    pub card: Option<Hotspot>,
    pub dirty: bool,
}

impl TooltipState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the dwell state by one frame. A `None` hover (no hotspot,
    /// pointer held, pointer gone) clears the card immediately.
    pub fn tick(&mut self, hover: Option<&Hotspot>, dt: Duration) -> TooltipFrame {
        let was_active = self.active;
        let previous_subject = self.subject.clone();
        match hover {
            Some(hotspot) if self.subject.as_ref() == Some(hotspot) => {
                if !self.active {
                    self.dwell += dt;
                    if self.dwell >= DWELL {
                        self.active = true;
                    }
                }
            }
            Some(hotspot) => {
                self.subject = Some(hotspot.clone());
                // The arrival frame's time already counts toward the dwell.
                self.dwell = dt;
                self.active = false;
            }
            None => {
                self.subject = None;
                self.dwell = Duration::ZERO;
                self.active = false;
            }
        }
        let card = if self.active {
            self.subject.clone()
        } else {
            None
        };
        // Keep ticking while a dwell is running, and force one frame on any
        // visibility or subject change so the card appears/disappears/moves
        // even in a demand-driven loop. A stable shown card is not dirty.
        let dirty = (hover.is_some() && !self.active)
            || was_active != self.active
            || (self.active && previous_subject != self.subject);
        TooltipFrame { card, dirty }
    }
}

/// Wrap text into lines of at most `max` columns, breaking on spaces; a
/// single word longer than `max` hard-breaks across lines.
pub fn wrap_lines(text: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;
    for word in text.split_whitespace().flat_map(|word| {
        // Hard-break words longer than the wrap width.
        let chars: Vec<char> = word.chars().collect();
        if chars.len() <= max {
            vec![word.to_string()]
        } else {
            chars
                .chunks(max)
                .map(|chunk| chunk.iter().collect())
                .collect()
        }
    }) {
        let word_len = word.chars().count();
        if !current.is_empty() && current_len + 1 + word_len > max {
            lines.push(std::mem::take(&mut current));
            current_len = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_len += 1;
        }
        current.push_str(&word);
        current_len += word_len;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

/// Internal card layout, resolved in Flat2d screen space where larger local
/// y renders higher on screen. `title_ys`/`description_ys` are the screen y
/// of each wrapped line, topmost (screen) line first.
struct CardLayout {
    card: ModuleRect,
    box_rect: ModuleRect,
    title_ys: Vec<i32>,
    description_ys: Vec<i32>,
}

fn expand_by_one(rect: ModuleRect) -> ModuleRect {
    ModuleRect {
        x0: rect.x0 - 1,
        y0: rect.y0 - 1,
        x1: rect.x1 + 1,
        y1: rect.y1 + 1,
    }
}

fn layout_card(hotspot: &Hotspot, screen: ModuleRect) -> CardLayout {
    let box_rect = expand_by_one(hotspot.rect);
    let title_lines = wrap_lines(&hotspot.title, TEXT_WRAP_COLUMNS);
    let description_lines = wrap_lines(&hotspot.description, TEXT_WRAP_COLUMNS);
    let title_h = title_lines.len() as i32;
    let description_h = description_lines.len() as i32;
    let text_columns = title_lines
        .iter()
        .chain(&description_lines)
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as i32;
    let box_columns = box_rect.x1 - box_rect.x0 + 1;
    let card_columns = box_columns.max(text_columns) + 2;

    let above = screen.y1 - box_rect.y1;
    let below = box_rect.y0 - screen.y0;

    // Preferred: title above the highlighted object, description below —
    // the framing card. Fallbacks keep the same framing but stack both text
    // blocks on whichever side has room, so the card stays visually
    // consistent at the screen's top/bottom edges.
    let fits_preferred = title_h < above && description_h < below;
    let fits_below = title_h + description_h + 2 < below;
    let fits_above = title_h + description_h + 2 < above;

    let (title_ys, description_ys, card_y0, card_y1) =
        if fits_preferred || (!fits_below && !fits_above) {
            (
                (0..title_h)
                    .map(|i| box_rect.y1 + title_h - i)
                    .collect::<Vec<_>>(),
                (0..description_h)
                    .map(|i| box_rect.y0 - 1 - i)
                    .collect::<Vec<_>>(),
                box_rect.y0 - description_h - 1,
                box_rect.y1 + title_h + 1,
            )
        } else if fits_below {
            (
                (0..title_h)
                    .map(|i| box_rect.y0 - 1 - i)
                    .collect::<Vec<_>>(),
                (0..description_h)
                    .map(|i| box_rect.y0 - 1 - title_h - i)
                    .collect::<Vec<_>>(),
                box_rect.y0 - 1 - title_h - description_h - 1,
                box_rect.y1 + 1,
            )
        } else {
            (
                (0..title_h)
                    .map(|i| box_rect.y1 + 1 + description_h + (title_h - 1 - i))
                    .collect::<Vec<_>>(),
                (0..description_h)
                    .map(|i| box_rect.y1 + 1 + i)
                    .collect::<Vec<_>>(),
                box_rect.y0,
                box_rect.y1 + 1 + description_h + title_h + 1 - 1,
            )
        };

    // Center the card on the highlighted object, then clamp into the screen.
    let box_center_x = (box_rect.x0 + box_rect.x1 + 1) / 2;
    let mut card_x0 = box_center_x - card_columns / 2;
    card_x0 = card_x0.max(screen.x0).min(screen.x1 - card_columns + 1);

    let (mut card_y0, mut card_y1) = (card_y0, card_y1);
    if card_y1 > screen.y1 {
        let shift = card_y1 - screen.y1;
        card_y0 -= shift;
        card_y1 -= shift;
    }
    if card_y0 < screen.y0 {
        let shift = screen.y0 - card_y0;
        card_y0 += shift;
        card_y1 += shift;
    }

    CardLayout {
        card: ModuleRect {
            x0: card_x0,
            y0: card_y0,
            x1: card_x0 + card_columns - 1,
            y1: card_y1,
        },
        box_rect,
        title_ys,
        description_ys,
    }
}

fn center_start_x(line_columns: usize, lo: i32, hi: i32) -> i32 {
    let span = hi - lo + 1;
    lo + ((span - line_columns as i32) / 2).max(0)
}

fn push_text(cells: &mut Vec<Cell>, line: &str, y: i32, lo: i32, hi: i32, color: crate::CellColor) {
    let start = center_start_x(line.chars().count(), lo, hi);
    for (column, glyph) in line.chars().enumerate() {
        cells.push(Cell {
            position: CellPoint {
                x: start + column as i32,
                y,
                z: TEXT_DEPTH,
            },
            graphic: CellGraphic::Glyph(glyph),
            color,
            weight: CellWeight::from_index_clamped(2),
            ..Cell::default()
        });
    }
}

/// The tooltip card for one hotspot: a solid backer with rounded outer
/// corners framing the highlighted object, the title centered above it,
/// the (wrapped) description centered below. The highlighted cells
/// themselves are never drawn — the owning module's glyph shows through.
pub fn tooltip_card_cells(hotspot: &Hotspot, screen: ModuleRect, palette: &UiPalette) -> Vec<Cell> {
    let layout = layout_card(hotspot, screen);
    let fill = palette.get(UiColorRole::Dimmest);
    let wall = palette.get(UiColorRole::Dimmest);
    let title_color = palette.get(UiColorRole::Bright);
    let description_color = palette.get(UiColorRole::Medium);

    let mut cells: Vec<Cell> = Vec::new();
    let push = |cells: &mut Vec<Cell>, x: i32, y: i32, z: i32, glyph: char, color, weight: i32| {
        cells.push(Cell {
            position: CellPoint { x, y, z },
            graphic: CellGraphic::Glyph(glyph),
            color,
            weight: CellWeight::from_index_clamped(weight),
            ..Cell::default()
        });
    };

    // Solid backer at z 0 (weight 3), minus the highlighted cells themselves
    // so the owning module's glyph shows through, minus the four card
    // corners so the rounded corner glyphs read against the screen, and
    // minus the ring cells so the ring glyphs replace the backer instead of
    // stacking over it.
    let (x0, x1, y0, y1) = (
        layout.card.x0,
        layout.card.x1,
        layout.card.y0,
        layout.card.y1,
    );
    let corners = [(x0, y1, '▟'), (x1, y1, '▙'), (x1, y0, '▛'), (x0, y0, '▜')];
    let box_r = layout.box_rect;
    let ring = |x: i32, y: i32| -> Option<char> {
        if x == box_r.x0 && y >= box_r.y0 && y <= box_r.y1 {
            Some('◧')
        } else if x == box_r.x1 && y >= box_r.y0 && y <= box_r.y1 {
            Some('◨')
        } else if y == box_r.y0 && x >= box_r.x0 && x <= box_r.x1 {
            Some('◪')
        } else if y == box_r.y1 && x >= box_r.x0 && x <= box_r.x1 {
            Some('◩')
        } else {
            None
        }
    };
    for y in layout.card.y0..=layout.card.y1 {
        for x in layout.card.x0..=layout.card.x1 {
            if hotspot.rect.contains(x, y)
                || corners.iter().any(|(cx, cy, _)| *cx == x && *cy == y)
                || ring(x, y).is_some()
            {
                continue;
            }
            push(&mut cells, x, y, 0, '█', fill, 3);
        }
    }

    // Rounded outer corners (weight 3), same color as the backer — they are
    // the backer's corners.
    for (x, y, glyph) in corners {
        push(&mut cells, x, y, 0, glyph, fill, 3);
    }

    // Overlay layer: text lives at TEXT_DEPTH above the backer, so the
    // backer stays valid behind it and picks up the perspective depth offset.

    // Inner walls framing the highlighted object — same color as the backer,
    // same z 0 layer, replacing the backer cells; the half-block glyphs fade
    // in toward the highlighted object.
    for x in box_r.x0..=box_r.x1 {
        push(&mut cells, x, box_r.y1, 0, '◩', wall, 3);
        push(&mut cells, x, box_r.y0, 0, '◪', wall, 3);
    }
    for y in (box_r.y0 + 1)..box_r.y1 {
        push(&mut cells, box_r.x0, y, 0, '◧', wall, 3);
        push(&mut cells, box_r.x1, y, 0, '◨', wall, 3);
    }

    // Centered text: title above (larger y), description below.
    let text_lo = layout.card.x0 + 1;
    let text_hi = layout.card.x1 - 1;
    for (i, line) in wrap_lines(&hotspot.title, TEXT_WRAP_COLUMNS)
        .into_iter()
        .enumerate()
    {
        push_text(
            &mut cells,
            &line,
            layout.title_ys[i],
            text_lo,
            text_hi,
            title_color,
        );
    }
    for (i, line) in wrap_lines(&hotspot.description, TEXT_WRAP_COLUMNS)
        .into_iter()
        .enumerate()
    {
        push_text(
            &mut cells,
            &line,
            layout.description_ys[i],
            text_lo,
            text_hi,
            description_color,
        );
    }

    cells
}

/// The composed overlay group the host pushes last so the card draws on top
/// of every module. Display-only: nothing here hit-tests or captures.
pub fn tooltip_card_group(hotspot: &Hotspot, screen: ModuleRect, palette: &UiPalette) -> CellGroup {
    CellGroup::from_cells(
        WorldPoint { x: 0, y: 0, z: 0 },
        tooltip_card_cells(hotspot, screen, palette),
    )
    .with_intake_behavior(CellGroupIntakeBehavior::Flat2d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellGraphic, UiPalette};

    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> ModuleRect {
        ModuleRect { x0, y0, x1, y1 }
    }

    fn hotspot_at(x: i32, y: i32) -> Hotspot {
        Hotspot::new(
            rect(x, y, x, y),
            "close",
            "closes the module can be re-opened",
        )
    }

    fn screen() -> ModuleRect {
        rect(0, 0, 100, 60)
    }

    #[test]
    fn wrap_breaks_on_spaces_at_the_limit() {
        let lines = wrap_lines("closes the module can be re-opened", 35);
        assert_eq!(lines, vec!["closes the module can be re-opened"]);
        let long = "click, then grab one of the four border edges around the panel to resize it";
        let lines = wrap_lines(long, 35);
        assert!(lines.len() >= 2);
        for line in &lines {
            assert!(line.chars().count() <= 35, "{line:?}");
        }
    }

    #[test]
    fn wrap_hard_breaks_a_word_longer_than_the_limit() {
        let lines = wrap_lines("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 35);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].chars().count(), 35);
    }

    #[test]
    fn dwell_fires_after_400ms_and_resets_on_subject_change() {
        let mut state = TooltipState::new();
        let hotspot = hotspot_at(10, 10);
        let other = hotspot_at(20, 20);

        assert_eq!(
            state.tick(Some(&hotspot), Duration::from_millis(399)).card,
            None
        );
        assert!(
            state.tick(Some(&hotspot), Duration::from_millis(1)).dirty,
            "dwell running keeps frames coming"
        );
        let shown = state.tick(Some(&hotspot), Duration::ZERO);
        assert_eq!(shown.card.as_ref(), Some(&hotspot));
        assert!(!shown.dirty, "stable shown card is not dirty");

        // Moving to another hotspot resets the dwell.
        assert_eq!(state.tick(Some(&other), Duration::ZERO).card, None);
        assert_eq!(
            state.tick(Some(&other), Duration::from_millis(399)).card,
            None
        );

        // Pointer gone clears immediately.
        assert_eq!(state.tick(None, Duration::ZERO).card, None);
    }

    #[test]
    fn preferred_layout_frames_the_object_title_above_description_below() {
        let hotspot = hotspot_at(50, 30);
        let layout = layout_card(&hotspot, screen());
        for title_y in &layout.title_ys {
            assert!(
                *title_y > layout.box_rect.y1,
                "title above the object on screen"
            );
        }
        for description_y in &layout.description_ys {
            assert!(
                *description_y < layout.box_rect.y0,
                "description below the object on screen"
            );
        }
        assert!(layout.card.contains(hotspot.rect.x0, hotspot.rect.y0));
    }

    #[test]
    fn card_at_the_screen_top_stacks_both_texts_below() {
        // Anchor on the screen's top row: no room above, so the whole card
        // content stacks below the framed object.
        let hotspot = Hotspot::new(
            rect(50, screen().y1, 50, screen().y1),
            "close",
            "closes the module can be re-opened",
        );
        let layout = layout_card(&hotspot, screen());
        for title_y in layout.title_ys.iter().chain(&layout.description_ys) {
            assert!(*title_y < layout.box_rect.y0, "text fell below the object");
        }
    }

    #[test]
    fn card_at_the_screen_bottom_stacks_both_texts_above() {
        let hotspot = Hotspot::new(
            rect(50, screen().y0, 50, screen().y0),
            "close",
            "closes the module can be re-opened",
        );
        let layout = layout_card(&hotspot, screen());
        for text_y in layout.title_ys.iter().chain(&layout.description_ys) {
            assert!(*text_y > layout.box_rect.y1, "text rose above the object");
        }
    }

    #[test]
    fn card_cells_frame_the_object_without_touching_it() {
        let hotspot = hotspot_at(50, 30);
        let cells = tooltip_card_cells(&hotspot, screen(), &UiPalette::default());
        let at = |x: i32, y: i32| {
            cells
                .iter()
                .rev()
                .find(|cell| cell.position.x == x && cell.position.y == y)
                .map(|cell| cell.graphic.clone())
        };

        // Rounded outer corners land on the card rect's corners.
        let layout = layout_card(&hotspot, screen());
        assert_eq!(
            at(layout.card.x0, layout.card.y1),
            Some(CellGraphic::Glyph('▟'))
        );
        assert_eq!(
            at(layout.card.x1, layout.card.y0),
            Some(CellGraphic::Glyph('▛'))
        );

        // Inner walls ring the object.
        assert_eq!(
            at(layout.box_rect.x0, layout.box_rect.y1),
            Some(CellGraphic::Glyph('◩'))
        );
        assert_eq!(
            at(layout.box_rect.x1, layout.box_rect.y0),
            Some(CellGraphic::Glyph('◪'))
        );

        // The highlighted cell itself is never drawn — the module shows through.
        assert_eq!(at(hotspot.rect.x0, hotspot.rect.y0), None);

        // Title and description text render somewhere in the card.
        let glyphs: String = cells
            .iter()
            .filter_map(|cell| match cell.graphic {
                CellGraphic::Glyph(glyph) => Some(glyph),
                _ => None,
            })
            .collect();
        assert!(glyphs.contains("close"), "{glyphs:?}");
        assert!(glyphs.contains("re-opened"), "{glyphs:?}");
    }

    #[test]
    fn card_group_is_flat2d_and_origin_anchored() {
        let hotspot = hotspot_at(50, 30);
        let group = tooltip_card_group(&hotspot, screen(), &UiPalette::default());
        assert_eq!((group.origin.x, group.origin.y, group.origin.z), (0, 0, 0));
        assert!(!group.cells.is_empty());
    }

    #[test]
    fn backer_is_solid_weight_three_blocks() {
        let hotspot = hotspot_at(50, 30);
        let cells = tooltip_card_cells(&hotspot, screen(), &UiPalette::default());
        let backer: Vec<_> = cells.iter().filter(|cell| cell.position.z == 0).collect();
        assert!(!backer.is_empty());
        for cell in &backer {
            match &cell.graphic {
                CellGraphic::Glyph('█')
                | CellGraphic::Glyph('▟')
                | CellGraphic::Glyph('▙')
                | CellGraphic::Glyph('▛')
                | CellGraphic::Glyph('▜')
                | CellGraphic::Glyph('◩')
                | CellGraphic::Glyph('◨')
                | CellGraphic::Glyph('◪')
                | CellGraphic::Glyph('◧') => {}
                other => panic!("unexpected z0 cell {other:?}"),
            }
            assert_eq!(cell.weight, CellWeight::Three);
        }
    }

    #[test]
    fn overlay_layers_in_depth_above_an_intact_backer() {
        let hotspot = hotspot_at(50, 30);
        let layout = layout_card(&hotspot, screen());
        let cells = tooltip_card_cells(&hotspot, screen(), &UiPalette::default());
        let at = |x: i32, y: i32, z: i32| {
            cells
                .iter()
                .find(|cell| cell.position.x == x && cell.position.y == y && cell.position.z == z)
                .map(|cell| cell.graphic.clone())
        };

        // Corners and ring are backer-layer cells at z 0 — no stacking, they
        // replace the █ instead of sitting over one.
        assert_eq!(
            at(layout.card.x0, layout.card.y1, 0),
            Some(CellGraphic::Glyph('▟'))
        );
        assert_eq!(at(layout.card.x0, layout.card.y1, TEXT_DEPTH), None);
        assert_eq!(
            at(layout.box_rect.x0, layout.box_rect.y1, 0),
            Some(CellGraphic::Glyph('◩'))
        );
        assert_eq!(at(layout.box_rect.x0, layout.box_rect.y1, TEXT_DEPTH), None);

        // Every overlay cell keeps the backer behind it — text/ring never
        // overwrite the backer, they layer above it.
        let backer_keys: Vec<_> = cells
            .iter()
            .filter(|cell| cell.position.z == 0)
            .map(|cell| (cell.position.x, cell.position.y))
            .collect();
        for cell in cells.iter().filter(|cell| cell.position.z == TEXT_DEPTH) {
            assert!(
                backer_keys.contains(&(cell.position.x, cell.position.y)),
                "overlay cell at {:?} has no backer behind it",
                cell.position
            );
        }
    }
}
