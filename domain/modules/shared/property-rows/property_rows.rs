use crate::{
    Cell, CellGraphic, CellPoint, CellWeight, Hotspot, ModulePointerButton, ModuleRect,
    PanelChrome, UiColorRole, UiPalette,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyMatrixColumn {
    pub id: String,
    pub label: String,
    pub left_value: bool,
    pub right_value: bool,
    pub left_enabled: bool,
    pub right_enabled: bool,
}

impl PropertyMatrixColumn {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            left_value: false,
            right_value: false,
            left_enabled: true,
            right_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyRow {
    Info {
        label: String,
        value: String,
    },
    Separator,
    Matrix {
        id: String,
        label: String,
        columns: Vec<PropertyMatrixColumn>,
        token_width: i32,
    },
    /// A row of small integer fields (one 2-char token each, signed), e.g. a
    /// 3-axis offset. Left-click a field to type a value in (the host drives
    /// [`NumberFieldEdit`] through its typing seam); scroll on a field to
    /// nudge it by one.
    NumberRow {
        id: String,
        label: String,
        values: Vec<i32>,
        min: i32,
        max: i32,
        /// The field currently being typed into: (field index, edit buffer).
        editing: Option<(usize, String)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyMatrixSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyHit {
    Matrix {
        row_id: String,
        side: PropertyMatrixSide,
        column_id: String,
    },
    /// A left click on one [`PropertyRow::NumberRow`] field.
    Number { row_id: String, field: usize },
}

/// Each number field renders as a signed 2-char token, then a 1-char gap.
const NUMBER_FIELD_WIDTH: i32 = 2;

/// In-place text-edit state for one [`PropertyRow::NumberRow`] field, driven
/// through the host program's typing seam: digits (and one leading `-`) edit
/// the buffer, Enter commits, Escape cancels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFieldEdit {
    pub row_id: String,
    pub field: usize,
    pub buffer: String,
}

impl NumberFieldEdit {
    pub fn begin(row_id: impl Into<String>, field: usize, current: i32) -> Self {
        Self {
            row_id: row_id.into(),
            field,
            buffer: current.to_string(),
        }
    }

    /// Accepts digits and one leading `-`; the buffer caps at 3 chars.
    pub fn push(&mut self, ch: char) {
        if !ch.is_ascii_digit() && ch != '-' {
            return;
        }
        if ch == '-' && !self.buffer.is_empty() {
            return;
        }
        if self.buffer.chars().count() >= 3 {
            return;
        }
        self.buffer.push(ch);
    }

    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    /// The parsed field value clamped to `min..max`, or `None` while the
    /// buffer is empty or not a number yet.
    pub fn commit(&self, min: i32, max: i32) -> Option<i32> {
        self.buffer
            .parse::<i32>()
            .ok()
            .map(|value| value.clamp(min, max))
    }
}

pub struct PropertyRows;

impl PropertyRows {
    const LABEL_WIDTH: i32 = 7;

    pub fn row_height(row: &PropertyRow) -> i32 {
        match row {
            PropertyRow::Separator => 1,
            PropertyRow::Info { .. } => 1,
            PropertyRow::Matrix { .. } => 1,
            PropertyRow::NumberRow { .. } => 1,
        }
    }

    pub fn top_row_y(rect: ModuleRect) -> i32 {
        let (_, content_y) = PanelChrome::content_origin();
        let (_, content_height) = PanelChrome::content_size(rect);
        content_y + (content_height - 1).max(0)
    }

    pub fn content_columns(rect: ModuleRect) -> (i32, i32, i32) {
        let (content_x, _) = PanelChrome::content_origin();
        let (content_width, _) = PanelChrome::content_size(rect);
        let label_x = content_x;
        let value_x = content_x + Self::LABEL_WIDTH + 1;
        let value_width = (content_width - (value_x - content_x)).max(1);
        (label_x, value_x, value_width)
    }

    pub fn draw(
        rect: ModuleRect,
        rows: &[PropertyRow],
        palette: &UiPalette,
        top_offset: i32,
    ) -> Vec<Cell> {
        let (label_x, value_x, value_width) = Self::content_columns(rect);
        let mut cells = Vec::new();
        // `top_offset` reserves whole rows at the top of the content area for
        // module-drawn header content (hand previews and the like); the row
        // list starts below them.
        let mut y = Self::top_row_y(rect) - top_offset;

        for row in rows {
            if y < PanelChrome::content_inset() {
                break;
            }
            match row {
                PropertyRow::Info { label, value } => {
                    for (index, glyph) in label.chars().enumerate() {
                        let x = label_x + index as i32;
                        if x >= value_x - 1 {
                            break;
                        }
                        cells.push(Cell {
                            position: CellPoint { x, y, z: 0 },
                            graphic: CellGraphic::Glyph(glyph),
                            color: palette.get(UiColorRole::Medium),
                            weight: CellWeight::from_index_clamped(1),
                            ..Cell::default()
                        });
                    }
                    for (index, glyph) in value.chars().enumerate() {
                        let x = value_x + index as i32;
                        if x >= value_x + value_width {
                            break;
                        }
                        cells.push(Cell {
                            position: CellPoint { x, y, z: 0 },
                            graphic: CellGraphic::Glyph(glyph),
                            color: palette.get(UiColorRole::Bright),
                            weight: CellWeight::from_index_clamped(2),
                            ..Cell::default()
                        });
                    }
                }
                PropertyRow::Separator => {
                    for x in label_x..(value_x + value_width) {
                        cells.push(Cell {
                            position: CellPoint { x, y, z: 0 },
                            graphic: CellGraphic::Glyph('─'),
                            color: palette.get(UiColorRole::Dimmest),
                            weight: CellWeight::from_index_clamped(1),
                            ..Cell::default()
                        });
                    }
                }
                PropertyRow::Matrix {
                    label,
                    columns,
                    token_width,
                    ..
                } => {
                    for (index, glyph) in label.chars().enumerate() {
                        let x = label_x + index as i32;
                        if x >= value_x - 1 {
                            break;
                        }
                        cells.push(Cell {
                            position: CellPoint { x, y, z: 0 },
                            graphic: CellGraphic::Glyph(glyph),
                            color: palette.get(UiColorRole::Medium),
                            weight: CellWeight::from_index_clamped(1),
                            ..Cell::default()
                        });
                    }
                    let column_count = columns.len().max(1) as i32;
                    let column_width = (*token_width).max(value_width / column_count).max(1);
                    for (index, column) in columns.iter().enumerate() {
                        let start_x = value_x + index as i32 * column_width;
                        let token: String =
                            column.label.chars().take(column_width as usize).collect();
                        let token_x =
                            start_x + ((column_width - token.chars().count() as i32).max(0) / 2);
                        let color = if !column.left_enabled && !column.right_enabled {
                            palette.get(UiColorRole::Dimmest)
                        } else if column.left_value && column.right_value {
                            palette.get(UiColorRole::Vivid)
                        } else if column.left_value {
                            palette.get(UiColorRole::LeftHand)
                        } else if column.right_value {
                            palette.get(UiColorRole::RightHand)
                        } else {
                            palette.get(UiColorRole::Bright)
                        };
                        let weight = if column.left_value || column.right_value {
                            2
                        } else {
                            1
                        };
                        for (offset, glyph) in token.chars().enumerate() {
                            let x = token_x + offset as i32;
                            if x >= value_x + value_width {
                                break;
                            }
                            cells.push(Cell {
                                position: CellPoint { x, y, z: 0 },
                                graphic: CellGraphic::Glyph(glyph),
                                color,
                                weight: CellWeight::from_index_clamped(weight),
                                ..Cell::default()
                            });
                        }
                    }
                }
                PropertyRow::NumberRow {
                    label,
                    values,
                    editing,
                    ..
                } => {
                    for (index, glyph) in label.chars().enumerate() {
                        let x = label_x + index as i32;
                        if x >= value_x - 1 {
                            break;
                        }
                        cells.push(Cell {
                            position: CellPoint { x, y, z: 0 },
                            graphic: CellGraphic::Glyph(glyph),
                            color: palette.get(UiColorRole::Medium),
                            weight: CellWeight::from_index_clamped(1),
                            ..Cell::default()
                        });
                    }
                    for (index, value) in values.iter().enumerate() {
                        let start_x = value_x + index as i32 * (NUMBER_FIELD_WIDTH + 1);
                        let (text, editing_field) = match editing {
                            Some((field, buffer)) if *field == index => {
                                (format!("{buffer}_"), true)
                            }
                            _ => (Self::signed_number(*value), false),
                        };
                        let color = if editing_field {
                            palette.get(UiColorRole::Vivid)
                        } else {
                            palette.get(UiColorRole::Bright)
                        };
                        for (offset, glyph) in text
                            .chars()
                            .take((NUMBER_FIELD_WIDTH + 1) as usize)
                            .enumerate()
                        {
                            cells.push(Cell {
                                position: CellPoint {
                                    x: start_x + offset as i32,
                                    y,
                                    z: 0,
                                },
                                graphic: CellGraphic::Glyph(glyph),
                                color,
                                weight: CellWeight::from_index_clamped(2),
                                ..Cell::default()
                            });
                        }
                    }
                }
            }
            y -= Self::row_height(row);
        }

        cells
    }

    /// The (row id, field index) of the [`PropertyRow::NumberRow`] field under
    /// `(x, y)`, if any — the scroll-to-nudge hit target.
    pub fn number_field_at(
        rect: ModuleRect,
        rows: &[PropertyRow],
        x: i32,
        y: i32,
        top_offset: i32,
    ) -> Option<(String, usize)> {
        let (_, value_x, _value_width) = Self::content_columns(rect);
        let local_y = y - rect.y0;
        let local_x = x - rect.x0;
        let mut row_y = Self::top_row_y(rect) - top_offset;

        for row in rows {
            let row_height = Self::row_height(row);
            if local_y > row_y || local_y <= row_y - row_height {
                row_y -= row_height;
                continue;
            }
            if let PropertyRow::NumberRow { id, values, .. } = row {
                for (index, _value) in values.iter().enumerate() {
                    let start_x = value_x + index as i32 * (NUMBER_FIELD_WIDTH + 1);
                    if local_x >= start_x && local_x < start_x + NUMBER_FIELD_WIDTH {
                        return Some((id.clone(), index));
                    }
                }
            }
            row_y -= row_height;
        }

        None
    }

    /// Signed 2-char token: `-9`..`+9`.
    fn signed_number(value: i32) -> String {
        if value < 0 {
            format!("{value}")
        } else {
            format!("+{value}")
        }
    }

    pub fn hit_test(
        rect: ModuleRect,
        rows: &[PropertyRow],
        x: i32,
        y: i32,
        button: ModulePointerButton,
        top_offset: i32,
    ) -> Option<PropertyHit> {
        let (_, value_x, value_width) = Self::content_columns(rect);
        let local_x = x - rect.x0;
        let local_y = y - rect.y0;
        let side = match button {
            ModulePointerButton::Left => PropertyMatrixSide::Left,
            ModulePointerButton::Right => PropertyMatrixSide::Right,
            ModulePointerButton::Middle => return None,
        };
        let mut row_y = Self::top_row_y(rect) - top_offset;

        for row in rows {
            let row_height = Self::row_height(row);
            if local_y > row_y || local_y <= row_y - row_height {
                row_y -= row_height;
                continue;
            }
            if let PropertyRow::Matrix {
                id,
                columns,
                token_width,
                ..
            } = row
            {
                let column_count = columns.len().max(1) as i32;
                let column_width = (*token_width).max(value_width / column_count).max(1);
                for (index, column) in columns.iter().enumerate() {
                    let start_x = value_x + index as i32 * column_width;
                    let token: String = column.label.chars().take(column_width as usize).collect();
                    let token_x =
                        start_x + ((column_width - token.chars().count() as i32).max(0) / 2);
                    let token_x1 = token_x + token.chars().count() as i32 - 1;
                    if local_x >= token_x && local_x <= token_x1 {
                        let enabled = match side {
                            PropertyMatrixSide::Left => column.left_enabled,
                            PropertyMatrixSide::Right => column.right_enabled,
                        };
                        if !enabled {
                            return None;
                        }
                        #[cfg(feature = "debug-hits")]
                        eprintln!(
                            "[hit-debug] row={} column={} side={:?}",
                            id, column.id, side
                        );
                        return Some(PropertyHit::Matrix {
                            row_id: id.clone(),
                            side,
                            column_id: column.id.clone(),
                        });
                    }
                }
            }
            if let PropertyRow::NumberRow { id, values, .. } = row {
                for (index, _value) in values.iter().enumerate() {
                    let start_x = value_x + index as i32 * (NUMBER_FIELD_WIDTH + 1);
                    if local_x >= start_x && local_x < start_x + NUMBER_FIELD_WIDTH {
                        return match button {
                            ModulePointerButton::Left => Some(PropertyHit::Number {
                                row_id: id.clone(),
                                field: index,
                            }),
                            _ => None,
                        };
                    }
                }
            }
            row_y -= row_height;
        }

        None
    }

    /// Tooltip hotspots for the visible property rows, mirroring `draw`'s
    /// iteration: one full-row hotspot per row with the row's label as title
    /// and a kind-appropriate interaction description. Consumed through the
    /// module's `Module::hotspots` override alongside its gizmo bar.
    pub fn hotspots(rect: ModuleRect, rows: &[PropertyRow], top_offset: i32) -> Vec<Hotspot> {
        let (content_x, _, _) = Self::content_columns(rect);
        let (_, content_width) = PanelChrome::content_size(rect);
        let x1 = rect.x0 + content_x + content_width - 1;
        let mut hotspots = Vec::new();
        let mut y = Self::top_row_y(rect) - top_offset;

        for row in rows {
            if y < PanelChrome::content_inset() {
                break;
            }
            let (title, description) = match row {
                PropertyRow::Separator => {
                    y -= Self::row_height(row);
                    continue;
                }
                PropertyRow::Info { label, value } => {
                    (label.clone(), format!("currently: {value}"))
                }
                PropertyRow::Matrix { label, .. } => (
                    label.clone(),
                    "click a token to set it: left-click is the left hand, right-click the right".to_string(),
                ),
                PropertyRow::NumberRow { label, .. } => (
                    label.clone(),
                    "click a field to type a value, scroll a field to nudge it".to_string(),
                ),
            };
            hotspots.push(Hotspot::new(
                ModuleRect {
                    x0: rect.x0 + content_x,
                    y0: rect.y0 + y,
                    x1,
                    y1: rect.y0 + y,
                },
                title,
                description,
            ));
            y -= Self::row_height(row);
        }

        hotspots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> ModuleRect {
        ModuleRect {
            x0: 10,
            y0: 20,
            x1: 40,
            y1: 28,
        }
    }

    #[test]
    fn draws_info_and_matrix_rows_inside_the_panel_content_area() {
        let rows = vec![
            PropertyRow::Info {
                label: "tool".into(),
                value: "Brush/Fill".into(),
            },
            PropertyRow::Matrix {
                id: "mask".into(),
                label: "mask".into(),
                columns: vec![
                    PropertyMatrixColumn::new("glyph", "gly"),
                    PropertyMatrixColumn::new("color", "col"),
                ],
                token_width: 4,
            },
        ];

        let _y = PropertyRows::top_row_y(rect());
        let cells = PropertyRows::draw(rect(), &rows, &UiPalette::default(), 0);
        assert!(cells
            .iter()
            .any(|cell| cell.position == CellPoint { x: 1, y: 5, z: 0 }));
        assert!(cells
            .iter()
            .any(|cell| matches!(cell.graphic, CellGraphic::Glyph('g'))));
    }

    #[test]
    fn hit_test_resolves_the_clicked_matrix_column_and_pointer_side() {
        let rows = vec![PropertyRow::Matrix {
            id: "mask".into(),
            label: "mask".into(),
            columns: vec![
                PropertyMatrixColumn::new("glyph", "gly"),
                PropertyMatrixColumn::new("color", "col"),
            ],
            token_width: 4,
        }];

        let hit = PropertyRows::hit_test(rect(), &rows, 23, 25, ModulePointerButton::Right, 0);
        assert_eq!(
            hit,
            Some(PropertyHit::Matrix {
                row_id: "mask".into(),
                side: PropertyMatrixSide::Right,
                column_id: "glyph".into(),
            })
        );
    }

    fn number_row() -> PropertyRow {
        PropertyRow::NumberRow {
            id: "step".into(),
            label: "char".into(),
            values: vec![1, 0, -9],
            min: -9,
            max: 9,
            editing: None,
        }
    }

    fn number_row_editing(field: usize, buffer: &str) -> Vec<PropertyRow> {
        vec![PropertyRow::NumberRow {
            id: "step".into(),
            label: "char".into(),
            values: vec![1, 0, -9],
            min: -9,
            max: 9,
            editing: Some((field, buffer.to_string())),
        }]
    }

    #[test]
    fn number_row_draws_each_field_as_a_signed_two_char_token() {
        let _y = PropertyRows::top_row_y(rect());
        let y = PropertyRows::top_row_y(rect());
        let cells = PropertyRows::draw(rect(), &[number_row()], &UiPalette::default(), 0);
        let glyphs = |x: i32| {
            cells
                .iter()
                .filter(|cell| cell.position.x == x && cell.position.y == y)
                .filter_map(|cell| match cell.graphic {
                    CellGraphic::Glyph(g) => Some(g),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .pop()
        };
        let (label_x, value_x, _) = PropertyRows::content_columns(rect());
        assert_eq!(glyphs(label_x), Some('c'));
        assert_eq!(glyphs(label_x + 1), Some('h'));
        assert_eq!(glyphs(value_x), Some('+'));
        assert_eq!(glyphs(value_x + 1), Some('1'));
        // Field 2 sits three cells right of field 1 (2-char field + 1 gap).
        assert_eq!(glyphs(value_x + 3), Some('+'));
        assert_eq!(glyphs(value_x + 4), Some('0'));
        assert_eq!(glyphs(value_x + 3 * 2), Some('-'));
        assert_eq!(glyphs(value_x + 3 * 2 + 1), Some('9'));
    }

    #[test]
    fn number_row_click_opens_a_field_and_the_edit_buffer_replaces_the_token() {
        let _y = PropertyRows::top_row_y(rect());
        let rows = number_row_editing(1, "-4");
        let y = PropertyRows::top_row_y(rect());
        let cells = PropertyRows::draw(rect(), &rows, &UiPalette::default(), 0);
        let (_, value_x, _) = PropertyRows::content_columns(rect());
        let mut field_1: Vec<(i32, char)> = cells
            .iter()
            .filter(|cell| cell.position.y == y)
            .filter(|cell| cell.position.x >= value_x + 3 && cell.position.x <= value_x + 5)
            .filter_map(|cell| match cell.graphic {
                CellGraphic::Glyph(g) => Some((cell.position.x, g)),
                _ => None,
            })
            .collect();
        field_1.sort();
        assert_eq!(
            field_1,
            vec![(value_x + 3, '-'), (value_x + 4, '4'), (value_x + 5, '_')]
        );
    }

    #[test]
    fn clicking_a_number_field_hits_that_field_and_other_buttons_do_not() {
        let _y = PropertyRows::top_row_y(rect());
        let rows = vec![number_row()];
        let (_, value_x, _) = PropertyRows::content_columns(rect());
        // hit_test takes screen coords: module-local token x + rect origin.
        let field_2_x = rect().x0 + value_x + 2 * 3;
        let field_y = rect().y0 + PropertyRows::top_row_y(rect());

        assert_eq!(
            PropertyRows::hit_test(rect(), &rows, field_2_x, field_y, ModulePointerButton::Left, 0),
            Some(PropertyHit::Number {
                row_id: "step".into(),
                field: 2,
            })
        );
        assert_eq!(
            PropertyRows::hit_test(
                rect(),
                &rows,
                field_2_x,
                field_y,
                ModulePointerButton::Right,
                0
            ),
            None
        );
    }

    #[test]
    fn number_field_at_resolves_the_field_under_a_point_for_scroll_nudges() {
        let _y = PropertyRows::top_row_y(rect());
        let rows = vec![number_row()];
        let (_, value_x, _) = PropertyRows::content_columns(rect());
        // number_field_at takes screen coords like hit_test: local token x + rect origin.
        let field_x = |index: i32| rect().x0 + value_x + index * 3;
        let field_y = rect().y0 + PropertyRows::top_row_y(rect());

        assert_eq!(
            PropertyRows::number_field_at(rect(), &rows, field_x(0) + 1, field_y, 0),
            Some(("step".into(), 0))
        );
        assert_eq!(
            PropertyRows::number_field_at(rect(), &rows, field_x(1), field_y, 0),
            Some(("step".into(), 1))
        );
        // The gap between fields is not a field.
        assert_eq!(
            PropertyRows::number_field_at(rect(), &rows, field_x(0) + 2, field_y, 0),
            None
        );
    }

    #[test]
    fn number_field_edit_pushes_digits_and_commits_clamped() {
        let mut edit = NumberFieldEdit::begin("step", 0, 3);
        edit.push('7');
        assert_eq!(edit.buffer, "37");
        edit.push('-');
        assert_eq!(edit.buffer, "37");
        edit.backspace();
        edit.backspace();
        edit.backspace();
        edit.push('-');
        edit.push('9');
        assert_eq!(edit.commit(-9, 9), Some(-9));
        edit.push('9');
        assert_eq!(edit.buffer, "-99");
        assert_eq!(edit.commit(-9, 9), Some(-9));
    }

    #[test]
    fn hotspots_cover_each_visible_row_with_its_label() {
        let rows = vec![
            PropertyRow::Matrix {
                id: "weight".into(),
                label: "weight".into(),
                columns: vec![PropertyMatrixColumn::new("2", "2")],
                token_width: 2,
            },
            PropertyRow::Separator,
            PropertyRow::Info {
                label: "tools".into(),
                value: "pen / pen".into(),
            },
        ];
        let hotspots = PropertyRows::hotspots(rect(), &rows, 0);
        // Separator generates nothing; the other rows keep draw order.
        assert_eq!(hotspots.len(), 2);
        assert_eq!(hotspots[0].title, "weight");
        assert_eq!(hotspots[1].title, "tools");
        assert!(hotspots[1].description.contains("pen / pen"));
        // Hotspot rects are absolute and single content rows tall.
        assert!(hotspots[0].rect.y0 >= rect().y0);
        assert_eq!(hotspots[0].rect.y0, hotspots[0].rect.y1);
    }
}
