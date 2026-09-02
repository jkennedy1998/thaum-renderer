use crate::{
    Cell, CellGraphic, CellPoint, CellWeight, ModulePointerButton, ModuleRect, PanelChrome,
    UiColorRole, UiPalette,
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
}

pub struct PropertyRows;

impl PropertyRows {
    const LABEL_WIDTH: i32 = 7;

    fn row_height(row: &PropertyRow) -> i32 {
        match row {
            PropertyRow::Separator => 1,
            PropertyRow::Info { .. } => 1,
            PropertyRow::Matrix { .. } => 1,
        }
    }

    fn top_row_y(rect: ModuleRect) -> i32 {
        let (_, content_y) = PanelChrome::content_origin();
        let (_, content_height) = PanelChrome::content_size(rect);
        content_y + (content_height - 1).max(0)
    }

    fn content_columns(rect: ModuleRect) -> (i32, i32, i32) {
        let (content_x, _) = PanelChrome::content_origin();
        let (content_width, _) = PanelChrome::content_size(rect);
        let label_x = content_x;
        let value_x = content_x + Self::LABEL_WIDTH + 1;
        let value_width = (content_width - (value_x - content_x)).max(1);
        (label_x, value_x, value_width)
    }

    pub fn draw(rect: ModuleRect, rows: &[PropertyRow], palette: &UiPalette) -> Vec<Cell> {
        let (label_x, value_x, value_width) = Self::content_columns(rect);
        let mut cells = Vec::new();
        let mut y = Self::top_row_y(rect);

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
            }
            y -= Self::row_height(row);
        }

        cells
    }

    pub fn hit_test(
        rect: ModuleRect,
        rows: &[PropertyRow],
        x: i32,
        y: i32,
        button: ModulePointerButton,
    ) -> Option<PropertyHit> {
        let (_, value_x, value_width) = Self::content_columns(rect);
        let local_x = x - rect.x0;
        let local_y = y - rect.y0;
        let side = match button {
            ModulePointerButton::Left => PropertyMatrixSide::Left,
            ModulePointerButton::Right => PropertyMatrixSide::Right,
            ModulePointerButton::Middle => return None,
        };
        let mut row_y = Self::top_row_y(rect);

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
                        return Some(PropertyHit::Matrix {
                            row_id: id.clone(),
                            side,
                            column_id: column.id.clone(),
                        });
                    }
                }
            }
            row_y -= row_height;
        }

        None
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

        let cells = PropertyRows::draw(rect(), &rows, &UiPalette::default());
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

        let hit = PropertyRows::hit_test(rect(), &rows, 23, 25, ModulePointerButton::Right);
        assert_eq!(
            hit,
            Some(PropertyHit::Matrix {
                row_id: "mask".into(),
                side: PropertyMatrixSide::Right,
                column_id: "glyph".into(),
            })
        );
    }
}
