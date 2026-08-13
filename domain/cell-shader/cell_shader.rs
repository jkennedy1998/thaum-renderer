use crate::{CellTexture, CellWarble, CellWeight, DataLanes, WorldPoint};

pub const CELL_SHADER_PASS: u32 = 0;
pub const CELL_SHADER_WEIGHT_SIN: u32 = 1;
pub const CELL_SHADER_WARBLE_DIAGONAL: u32 = 3;
pub const CELL_SHADER_TEXTURE_SHIMMER: u32 = 5;
pub const CELL_SHADER_WARBLE_FUDGE_1: u32 = 6;
pub const CELL_SHADER_WARBLE_DISTORT_1: u32 = 7;
pub const CELL_SHADER_WARBLE_FUDGE_5: u32 = 9;
pub const CELL_SHADER_WARBLE_DISTORT_5: u32 = 10;

pub fn resolve_shaded_weight(
    base_weight: CellWeight,
    shader_stack: &[u32],
    world: WorldPoint,
    data_lanes: DataLanes,
) -> CellWeight {
    let mut weight = base_weight;

    for shader in shader_stack {
        match *shader {
            CELL_SHADER_PASS => {}
            CELL_SHADER_WEIGHT_SIN => {
                weight = apply_weight_sin(weight, world, data_lanes.breath().unwrap_or(0));
            }
            _ => {}
        }
    }

    weight
}

pub fn resolve_shaded_texture(
    base_texture: CellTexture,
    shader_stack: &[u32],
    world: WorldPoint,
    data_lanes: DataLanes,
) -> CellTexture {
    let mut texture = base_texture;

    for shader in shader_stack {
        match *shader {
            CELL_SHADER_PASS
            | CELL_SHADER_WEIGHT_SIN
            | CELL_SHADER_WARBLE_DIAGONAL
            | CELL_SHADER_WARBLE_FUDGE_1
            | CELL_SHADER_WARBLE_DISTORT_1
            | CELL_SHADER_WARBLE_FUDGE_5
            | CELL_SHADER_WARBLE_DISTORT_5 => {}
            CELL_SHADER_TEXTURE_SHIMMER => {
                texture = apply_texture_shimmer(world, data_lanes.breath().unwrap_or(0));
            }
            _ => {}
        }
    }

    texture
}

pub fn resolve_shaded_warble(
    base_warble: CellWarble,
    shader_stack: &[u32],
    world: WorldPoint,
    data_lanes: DataLanes,
) -> CellWarble {
    let mut warble = base_warble;

    for shader in shader_stack {
        match *shader {
            CELL_SHADER_PASS | CELL_SHADER_WEIGHT_SIN | CELL_SHADER_TEXTURE_SHIMMER => {}
            CELL_SHADER_WARBLE_DIAGONAL => {
                warble = apply_warble_diagonal(world, data_lanes.breath().unwrap_or(0));
            }
            CELL_SHADER_WARBLE_FUDGE_1 => {
                warble = apply_warble_fudge(world, data_lanes.breath().unwrap_or(0), 1);
            }
            CELL_SHADER_WARBLE_DISTORT_1 => {
                warble = apply_warble_distort(world, data_lanes.breath().unwrap_or(0), 1);
            }
            CELL_SHADER_WARBLE_FUDGE_5 => {
                warble = apply_warble_fudge(world, data_lanes.breath().unwrap_or(0), 5);
            }
            CELL_SHADER_WARBLE_DISTORT_5 => {
                warble = apply_warble_distort(world, data_lanes.breath().unwrap_or(0), 5);
            }
            _ => {}
        }
    }

    warble
}

fn apply_weight_sin(base_weight: CellWeight, world: WorldPoint, breath: i32) -> CellWeight {
    let phase = (breath + world.x + world.y + world.z).rem_euclid(4);
    let relative = match phase {
        0 => -1,
        1 => 0,
        2 => 1,
        3 => 0,
        _ => unreachable!(),
    };

    CellWeight::from_index_clamped(base_weight.as_index() as i32 + relative)
}

fn apply_texture_shimmer(_world: WorldPoint, _breath: i32) -> CellTexture {
    CellTexture::new(1)
}

fn apply_warble_fudge(world: WorldPoint, breath: i32, amount: u8) -> CellWarble {
    let phase = (breath + (world.x * 2) + world.y + world.z).rem_euclid(8) as u8;
    CellWarble::new(amount.saturating_mul(24).saturating_add(phase).max(1))
}

fn apply_warble_distort(world: WorldPoint, breath: i32, amount: u8) -> CellWarble {
    let phase = (breath + (world.x * 2) + world.y - world.z).rem_euclid(8) as u8;
    CellWarble::new(amount.saturating_mul(32).saturating_add(phase).max(1))
}

fn apply_warble_diagonal(world: WorldPoint, breath: i32) -> CellWarble {
    let phase = (breath + (world.x * 2) + world.y - world.z).rem_euclid(4) as u8;
    let code = match phase {
        0 => 48,
        1 => 96,
        2 => 144,
        3 => 192,
        _ => unreachable!(),
    };

    CellWarble::new(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_shader_keeps_weight_unchanged() {
        assert_eq!(
            resolve_shaded_weight(
                CellWeight::Two,
                &[CELL_SHADER_PASS],
                WorldPoint::origin(),
                DataLanes::with_breath(0),
            ),
            CellWeight::Two
        );
    }

    #[test]
    fn weight_sin_follows_the_stepping_minus_one_zero_plus_one_zero_pattern() {
        let world = WorldPoint::origin();
        let base = CellWeight::Two;

        assert_eq!(apply_weight_sin(base, world, 0), CellWeight::One);
        assert_eq!(apply_weight_sin(base, world, 1), CellWeight::Two);
        assert_eq!(apply_weight_sin(base, world, 2), CellWeight::Three);
        assert_eq!(apply_weight_sin(base, world, 3), CellWeight::Two);
    }

    #[test]
    fn later_shaders_win_for_weight_writes() {
        assert_eq!(
            resolve_shaded_weight(
                CellWeight::Two,
                &[CELL_SHADER_PASS, CELL_SHADER_WEIGHT_SIN],
                WorldPoint::origin(),
                DataLanes::with_breath(2),
            ),
            CellWeight::Three
        );
    }

    #[test]
    fn resolve_shaded_texture_keeps_base_texture_without_texture_shader() {
        assert_eq!(
            resolve_shaded_texture(
                CellTexture::new(33),
                &[CELL_SHADER_PASS, CELL_SHADER_WARBLE_DIAGONAL],
                WorldPoint::origin(),
                DataLanes::with_breath(2),
            ),
            CellTexture::new(33)
        );
    }

    #[test]
    fn resolve_shaded_texture_overrides_base_texture() {
        assert_eq!(
            resolve_shaded_texture(
                CellTexture::new(33),
                &[CELL_SHADER_TEXTURE_SHIMMER],
                WorldPoint::origin(),
                DataLanes::with_breath(2),
            ),
            CellTexture::new(1)
        );
    }

    #[test]
    fn warble_fudge_emits_nonzero_code() {
        let warble = apply_warble_fudge(WorldPoint { x: 1, y: 0, z: 0 }, 2, 5);
        assert_eq!(warble.code(), 124);
    }

    #[test]
    fn warble_diagonal_emits_breath_shifted_code() {
        let warble = apply_warble_diagonal(WorldPoint { x: 1, y: 0, z: 0 }, 2);
        assert_eq!(warble.code(), 48);
    }

    #[test]
    fn resolve_shaded_warble_keeps_base_warble_without_warble_shader() {
        assert_eq!(
            resolve_shaded_warble(
                CellWarble::new(77),
                &[CELL_SHADER_PASS],
                WorldPoint::origin(),
                DataLanes::with_breath(2),
            ),
            CellWarble::new(77)
        );
    }

    #[test]
    fn resolve_shaded_warble_applies_fudge_warble_5() {
        let warble = resolve_shaded_warble(
            CellWarble::none(),
            &[CELL_SHADER_WARBLE_FUDGE_5],
            WorldPoint::origin(),
            DataLanes::with_breath(2),
        );
        assert_eq!(warble.code(), 122);
    }

    #[test]
    fn resolve_shaded_warble_applies_diagonal_warble() {
        let warble = resolve_shaded_warble(
            CellWarble::none(),
            &[CELL_SHADER_WARBLE_DIAGONAL],
            WorldPoint::origin(),
            DataLanes::with_breath(2),
        );
        assert_eq!(warble.code(), 144);
    }
}
